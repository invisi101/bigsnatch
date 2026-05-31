use std::collections::BTreeSet;
use std::net::IpAddr;
use std::time::SystemTime;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrafficZone {
    External,
    Internal,
}

#[derive(Debug, Clone)]
pub struct DestinationSummary {
    pub process_name: String,
    pub display_dest: String,
    pub dst_addr: String,
    pub domain: String,
    pub connection_count: usize,
    pub ports: BTreeSet<u32>,
    pub protocols: BTreeSet<String>,
    pub first_seen: SystemTime,
    pub last_seen: SystemTime,
    pub zone: TrafficZone,
    pub connection_ids: Vec<u64>,
}

pub fn classify_zone(addr: &str) -> TrafficZone {
    let ip: IpAddr = match addr.parse() {
        Ok(ip) => ip,
        Err(_) => return TrafficZone::External,
    };

    match ip {
        IpAddr::V4(v4) => {
            if v4.is_loopback() || v4.is_private() || v4.is_link_local() || v4.is_unspecified() {
                TrafficZone::Internal
            } else {
                TrafficZone::External
            }
        }
        IpAddr::V6(v6) => {
            // Unwrap IPv4-mapped IPv6 (::ffff:192.168.x.x) and re-classify as IPv4
            if let Some(v4) = v6.to_ipv4_mapped() {
                return if v4.is_loopback() || v4.is_private() || v4.is_link_local() || v4.is_unspecified() {
                    TrafficZone::Internal
                } else {
                    TrafficZone::External
                };
            }
            // ULA (fc00::/7) — Rust stdlib has no built-in method
            let is_ula = (v6.segments()[0] & 0xfe00) == 0xfc00;
            if v6.is_loopback()
                || v6.is_unspecified()
                || v6.is_unicast_link_local()
                || v6.is_multicast()
                || is_ula
            {
                TrafficZone::Internal
            } else {
                TrafficZone::External
            }
        }
    }
}
