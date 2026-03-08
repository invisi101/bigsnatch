use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use anyhow::{Context, Result};
use aya::maps::RingBuf;
use aya::Ebpf;
use tokio::sync::broadcast;
use tokio::time::{interval, Duration};
use tracing::{debug, info, warn};

use snitchster_common::{ConnectionEvent as BpfConnectionEvent, DnsEvent as BpfDnsEvent};

use crate::dns_cache::DnsCache;
use crate::dns_parser;
use crate::process_cache::ProcessCache;
use crate::proto;

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

/// Run the main event processing loop.
/// Polls eBPF ring buffers at high frequency for connection and DNS events.
pub async fn run(
    ebpf: &mut Ebpf,
    process_cache: Arc<ProcessCache>,
    dns_cache: Arc<DnsCache>,
    event_tx: broadcast::Sender<proto::ConnectionEvent>,
) -> Result<()> {
    let events_map = ebpf
        .take_map("EVENTS")
        .context("EVENTS map not found")?;
    let mut events_rb = RingBuf::try_from(events_map)?;

    let dns_map = ebpf
        .take_map("DNS_EVENTS")
        .context("DNS_EVENTS map not found")?;
    let mut dns_rb = RingBuf::try_from(dns_map)?;

    info!("Event processor started, polling ring buffers");

    // Poll at 1ms intervals for low latency
    let mut poll_interval = interval(Duration::from_millis(1));

    loop {
        poll_interval.tick().await;

        // Drain DNS events FIRST so the cache is populated before we enrich connections
        while let Some(item) = dns_rb.next() {
            let data: &[u8] = &item;
            if data.len() < std::mem::size_of::<BpfDnsEvent>() {
                continue;
            }

            let dns_event: BpfDnsEvent =
                unsafe { std::ptr::read_unaligned(data.as_ptr() as *const BpfDnsEvent) };

            let len = dns_event.len as usize;
            if len > 0 && len <= 512 {
                if let Some(response) = dns_parser::parse_dns_response(&dns_event.data[..len]) {
                    for (ip, ttl) in &response.addresses {
                        dns_cache.insert(*ip, response.domain.clone(), *ttl);
                        debug!("DNS: {} → {}", response.domain, ip);
                    }
                }
            }
        }

        // Now drain connection events — DNS cache is up-to-date
        while let Some(item) = events_rb.next() {
            let data: &[u8] = &item;
            if data.len() < std::mem::size_of::<BpfConnectionEvent>() {
                warn!("Short event: {} bytes", data.len());
                continue;
            }

            let bpf_event: BpfConnectionEvent =
                unsafe { std::ptr::read_unaligned(data.as_ptr() as *const BpfConnectionEvent) };

            let enriched = enrich_event(&bpf_event, &process_cache, &dns_cache);

            debug!(
                "{} [{}] → {}:{} ({})",
                enriched.process_name,
                enriched.pid,
                if enriched.domain.is_empty() { &enriched.dst_addr } else { &enriched.domain },
                enriched.dst_port,
                if enriched.protocol == proto::Protocol::Tcp as i32 { "TCP" } else { "UDP" }
            );

            let _ = event_tx.send(enriched);
        }
    }
}

fn enrich_event(
    bpf_event: &BpfConnectionEvent,
    process_cache: &ProcessCache,
    dns_cache: &DnsCache,
) -> proto::ConnectionEvent {
    let comm = std::str::from_utf8(&bpf_event.comm)
        .unwrap_or("")
        .trim_end_matches('\0')
        .to_string();

    let proc_info = process_cache.get_or_lookup(bpf_event.pid, bpf_event.uid, &comm);

    let (src_addr, dst_addr) = if bpf_event.ip_version == 4 {
        let src = Ipv4Addr::new(
            bpf_event.src_addr[0], bpf_event.src_addr[1],
            bpf_event.src_addr[2], bpf_event.src_addr[3],
        );
        let dst = Ipv4Addr::new(
            bpf_event.dst_addr[0], bpf_event.dst_addr[1],
            bpf_event.dst_addr[2], bpf_event.dst_addr[3],
        );
        (IpAddr::V4(src), IpAddr::V4(dst))
    } else {
        let src = Ipv6Addr::from(bpf_event.src_addr);
        let dst = Ipv6Addr::from(bpf_event.dst_addr);
        (IpAddr::V6(src), IpAddr::V6(dst))
    };

    let domain = dns_cache.lookup(&dst_addr).unwrap_or_else(|| {
        // Fallback: try system reverse DNS lookup for IPs we missed
        // Skip private/loopback addresses — no point in reverse-looking those up
        let skip = match dst_addr {
            IpAddr::V4(v4) => v4.is_loopback() || v4.is_private() || v4.is_link_local(),
            IpAddr::V6(v6) => v6.is_loopback(),
        };
        if skip {
            return String::new();
        }
        // Quick blocking reverse lookup — acceptable since we're already in a tight loop
        match dns_lookup::lookup_addr(&dst_addr) {
            Ok(host) if host != dst_addr.to_string() => {
                // Cache it so we don't look it up again
                dns_cache.insert(dst_addr, host.clone(), 300);
                host
            }
            _ => String::new(),
        }
    });

    let protocol = if bpf_event.protocol == 6 {
        proto::Protocol::Tcp as i32
    } else {
        proto::Protocol::Udp as i32
    };

    let ip_version = if bpf_event.ip_version == 4 {
        proto::IpVersion::V4 as i32
    } else {
        proto::IpVersion::V6 as i32
    };

    proto::ConnectionEvent {
        id: NEXT_ID.fetch_add(1, Ordering::Relaxed),
        timestamp_ns: bpf_event.timestamp_ns,
        pid: bpf_event.pid,
        process_name: proc_info.name,
        exe_path: proc_info.exe_path,
        cmdline: proc_info.cmdline,
        uid: bpf_event.uid,
        username: proc_info.username,
        protocol,
        src_addr: src_addr.to_string(),
        src_port: bpf_event.src_port as u32,
        dst_addr: dst_addr.to_string(),
        dst_port: bpf_event.dst_port as u32,
        domain,
        ip_version,
    }
}
