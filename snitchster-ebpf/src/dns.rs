// DNS capture via socket filter.
//
// Attaches to a raw UDP socket to capture DNS response packets (source port 53).
// Raw packet data is forwarded to userspace via ring buffer for full parsing.
//
// DnsEvent is too large for the BPF stack (512-byte limit), so we use a
// per-CPU array map as scratch space.

use aya_ebpf::{
    helpers::bpf_ktime_get_ns,
    macros::socket_filter,
    programs::SkBuffContext,
};
use snitchster_common::DnsEvent;

use crate::{DNS_EVENTS, DNS_SCRATCH};

const ETH_HLEN: usize = 14;
const IP_PROTO_OFF: usize = ETH_HLEN + 9;
const IP_HLEN: usize = 20;
const UDP_SPORT_OFF: usize = ETH_HLEN + IP_HLEN;
const UDP_LEN_OFF: usize = ETH_HLEN + IP_HLEN + 4;
const UDP_HLEN: usize = 8;
const DNS_OFFSET: usize = ETH_HLEN + IP_HLEN + UDP_HLEN;

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
    // Check IP protocol is UDP
    let mut proto_buf = [0u8; 1];
    ctx.load_bytes(IP_PROTO_OFF, &mut proto_buf)
        .map_err(|_| 0i64)?;
    if proto_buf[0] != IPPROTO_UDP {
        return Ok(0);
    }

    // Check source port is 53 (DNS response)
    let mut sport_buf = [0u8; 2];
    ctx.load_bytes(UDP_SPORT_OFF, &mut sport_buf)
        .map_err(|_| 0i64)?;
    let sport = u16::from_be_bytes(sport_buf);
    if sport != DNS_PORT {
        return Ok(0);
    }

    // Get UDP payload length
    let mut ulen_buf = [0u8; 2];
    ctx.load_bytes(UDP_LEN_OFF, &mut ulen_buf)
        .map_err(|_| 0i64)?;
    let udp_len = u16::from_be_bytes(ulen_buf) as usize;

    let dns_len = if udp_len > UDP_HLEN {
        udp_len - UDP_HLEN
    } else {
        return Ok(0);
    };

    let copy_len = if dns_len > 512 { 512 } else { dns_len };

    // Use per-CPU scratch space instead of stack allocation
    let idx: u32 = 0;
    let event = unsafe { DNS_SCRATCH.get_ptr_mut(idx).ok_or(0i64)?.as_mut().ok_or(0i64)? };

    event.pid = 0;
    event.len = copy_len as u32;
    event.timestamp_ns = unsafe { bpf_ktime_get_ns() };

    // Zero the data buffer
    // BPF verifier needs bounded loops, so zero in chunks
    let mut i = 0;
    while i < 512 {
        event.data[i] = 0;
        i += 1;
    }

    // Copy DNS payload from packet
    if copy_len > 0 && copy_len <= 512 {
        ctx.load_bytes(DNS_OFFSET, &mut event.data[..copy_len])
            .map_err(|_| 0i64)?;
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
