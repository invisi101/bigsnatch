// DNS capture via socket filter.
//
// Attaches to a raw AF_PACKET socket to capture DNS response packets (source port 53).
// Raw packet data is forwarded to userspace via ring buffer for full parsing.
//
// DnsEvent is too large for the BPF stack (512-byte limit), so we use a
// per-CPU array map as scratch space.

use aya_ebpf::{
    helpers::{bpf_ktime_get_ns, bpf_skb_load_bytes},
    macros::socket_filter,
    programs::SkBuffContext,
};
use snitchster_common::DnsEvent;

use crate::{DNS_EVENTS, DNS_SCRATCH};

const ETH_HLEN: u32 = 14;
const IP_PROTO_OFF: u32 = ETH_HLEN + 9;
const IP_HLEN: u32 = 20;
const UDP_SPORT_OFF: u32 = ETH_HLEN + IP_HLEN;
const UDP_LEN_OFF: u32 = ETH_HLEN + IP_HLEN + 4;
const UDP_HLEN: u32 = 8;
const DNS_OFFSET: u32 = ETH_HLEN + IP_HLEN + UDP_HLEN;

const IPPROTO_UDP: u8 = 17;
const DNS_PORT: u16 = 53;

#[socket_filter]
pub fn dns_filter(ctx: SkBuffContext) -> i64 {
    match try_dns_filter(&ctx) {
        Ok(action) => action,
        Err(_) => 0,
    }
}

fn try_dns_filter(ctx: &SkBuffContext) -> Result<i64, i64> {
    let skb = ctx.skb.skb;

    // Check IP protocol is UDP — use raw bpf_skb_load_bytes helper directly
    let mut proto_buf = [0u8; 4];
    let ret = unsafe {
        bpf_skb_load_bytes(skb as *const _, IP_PROTO_OFF, proto_buf.as_mut_ptr() as *mut _, 4)
    };
    if ret != 0 {
        return Ok(0);
    }
    if proto_buf[0] != IPPROTO_UDP {
        return Ok(0);
    }

    // Check source port is 53 (DNS response)
    let mut sport_buf = [0u8; 4];
    let ret = unsafe {
        bpf_skb_load_bytes(skb as *const _, UDP_SPORT_OFF, sport_buf.as_mut_ptr() as *mut _, 4)
    };
    if ret != 0 {
        return Ok(0);
    }
    let sport = u16::from_be_bytes([sport_buf[0], sport_buf[1]]);
    if sport != DNS_PORT {
        return Ok(0);
    }

    // Get UDP payload length
    let mut ulen_buf = [0u8; 4];
    let ret = unsafe {
        bpf_skb_load_bytes(skb as *const _, UDP_LEN_OFF, ulen_buf.as_mut_ptr() as *mut _, 4)
    };
    if ret != 0 {
        return Ok(0);
    }
    let udp_len = u16::from_be_bytes([ulen_buf[0], ulen_buf[1]]) as u32;

    if udp_len <= UDP_HLEN {
        return Ok(0);
    }
    let dns_len = udp_len - UDP_HLEN;
    let copy_len = if dns_len > 512 { 512u32 } else { dns_len };

    // Use per-CPU scratch space instead of stack allocation
    let idx: u32 = 0;
    let event = unsafe { crate::DNS_SCRATCH.get_ptr_mut(idx).ok_or(0i64)?.as_mut().ok_or(0i64)? };

    event.pid = 0;
    event.len = copy_len;
    event.timestamp_ns = unsafe { bpf_ktime_get_ns() };

    // Zero the data buffer
    let mut i = 0;
    while i < 512 {
        event.data[i] = 0;
        i += 1;
    }

    // Copy DNS payload — use fixed 512-byte read, we've already zeroed the buffer
    // The verifier needs a compile-time-known constant size
    if copy_len > 0 {
        let read_len = if copy_len > 512 { 512u32 } else { copy_len };
        // Read up to 512 bytes — we always read exactly 512 to keep verifier happy
        // (extra bytes beyond dns_len are zeroed and ignored by userspace via event.len)
        let ret = unsafe {
            bpf_skb_load_bytes(
                skb as *const _,
                DNS_OFFSET,
                event.data.as_mut_ptr() as *mut _,
                512,
            )
        };
        // If the packet is shorter than 512, try with actual length
        if ret != 0 && read_len < 512 {
            // Fallback: try reading just what we need in known-size chunks
            // Read 256 bytes first
            if read_len >= 256 {
                let _ = unsafe {
                    bpf_skb_load_bytes(
                        skb as *const _,
                        DNS_OFFSET,
                        event.data.as_mut_ptr() as *mut _,
                        256,
                    )
                };
            } else if read_len >= 128 {
                let _ = unsafe {
                    bpf_skb_load_bytes(
                        skb as *const _,
                        DNS_OFFSET,
                        event.data.as_mut_ptr() as *mut _,
                        128,
                    )
                };
            } else if read_len >= 64 {
                let _ = unsafe {
                    bpf_skb_load_bytes(
                        skb as *const _,
                        DNS_OFFSET,
                        event.data.as_mut_ptr() as *mut _,
                        64,
                    )
                };
            }
        }
    }

    // Submit to ring buffer
    if let Some(mut buf) = DNS_EVENTS.reserve::<DnsEvent>(0) {
        unsafe {
            core::ptr::write(buf.as_mut_ptr(), *event);
        }
        buf.submit(0);
    }

    Ok(ctx.len() as i64)
}
