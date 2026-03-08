use aya_ebpf::{
    helpers::{
        bpf_get_current_comm, bpf_get_current_pid_tgid, bpf_get_current_uid_gid,
        bpf_ktime_get_ns, bpf_probe_read_kernel,
    },
    macros::kprobe,
    programs::ProbeContext,
};
use snitchster_common::ConnectionEvent;

use crate::EVENTS;

const SKC_FAMILY_OFF: usize = 16;
const SKC_DADDR_OFF: usize = 0;
const SKC_RCVSADDR_OFF: usize = 4;
const SKC_DPORT_OFF: usize = 12;
const SKC_NUM_OFF: usize = 14;
const SKC_V6_DADDR_OFF: usize = 40;
const SKC_V6_RCVSADDR_OFF: usize = 56;

const AF_INET: u16 = 2;
const AF_INET6: u16 = 10;

#[kprobe]
pub fn kprobe__udp_sendmsg(ctx: ProbeContext) -> u32 {
    match try_udp_sendmsg(&ctx) {
        Ok(()) => 0,
        Err(_) => 0,
    }
}

#[kprobe]
pub fn kprobe__udpv6_sendmsg(ctx: ProbeContext) -> u32 {
    match try_udp_sendmsg(&ctx) {
        Ok(()) => 0,
        Err(_) => 0,
    }
}

fn try_udp_sendmsg(ctx: &ProbeContext) -> Result<(), i64> {
    let sk: u64 = ctx.arg(0).ok_or(1i64)?;
    let sock_ptr = sk as *const u8;

    let pid_tgid = bpf_get_current_pid_tgid();
    let pid = (pid_tgid >> 32) as u32;
    let uid = bpf_get_current_uid_gid() as u32;
    let comm = bpf_get_current_comm().map_err(|_| 1i64)?;
    let ts = unsafe { bpf_ktime_get_ns() };

    let family: u16 = unsafe {
        bpf_probe_read_kernel(sock_ptr.add(SKC_FAMILY_OFF) as *const u16).map_err(|_| 1i64)?
    };

    let mut event = ConnectionEvent {
        event_type: 2, // UDP
        ip_version: 4,
        protocol: 17, // IPPROTO_UDP
        _pad: 0,
        pid,
        uid,
        src_addr: [0u8; 16],
        dst_addr: [0u8; 16],
        src_port: 0,
        dst_port: 0,
        comm,
        timestamp_ns: ts,
    };

    if family == AF_INET {
        let src: u32 = unsafe {
            bpf_probe_read_kernel(sock_ptr.add(SKC_RCVSADDR_OFF) as *const u32)
                .map_err(|_| 1i64)?
        };
        let dst: u32 = unsafe {
            bpf_probe_read_kernel(sock_ptr.add(SKC_DADDR_OFF) as *const u32)
                .map_err(|_| 1i64)?
        };
        event.src_addr[..4].copy_from_slice(&src.to_ne_bytes());
        event.dst_addr[..4].copy_from_slice(&dst.to_ne_bytes());
        event.ip_version = 4;
    } else if family == AF_INET6 {
        let src: [u8; 16] = unsafe {
            bpf_probe_read_kernel(sock_ptr.add(SKC_V6_RCVSADDR_OFF) as *const [u8; 16])
                .map_err(|_| 1i64)?
        };
        let dst: [u8; 16] = unsafe {
            bpf_probe_read_kernel(sock_ptr.add(SKC_V6_DADDR_OFF) as *const [u8; 16])
                .map_err(|_| 1i64)?
        };
        event.src_addr = src;
        event.dst_addr = dst;
        event.ip_version = 6;
    } else {
        return Ok(()); // Skip unknown address families
    }

    let dport_be: u16 = unsafe {
        bpf_probe_read_kernel(sock_ptr.add(SKC_DPORT_OFF) as *const u16).map_err(|_| 1i64)?
    };
    let sport: u16 = unsafe {
        bpf_probe_read_kernel(sock_ptr.add(SKC_NUM_OFF) as *const u16).map_err(|_| 1i64)?
    };

    event.dst_port = u16::from_be(dport_be);
    event.src_port = sport;

    // Don't emit events with no destination (unconnected sockets sending nowhere)
    let all_zero = event.dst_addr.iter().all(|&b| b == 0);
    if all_zero && event.dst_port == 0 {
        return Ok(());
    }

    if let Some(mut buf) = EVENTS.reserve::<ConnectionEvent>(0) {
        unsafe {
            core::ptr::write(buf.as_mut_ptr(), event);
        }
        buf.submit(0);
    }

    Ok(())
}
