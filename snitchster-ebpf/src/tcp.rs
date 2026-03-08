use aya_ebpf::{
    helpers::{
        bpf_get_current_comm, bpf_get_current_pid_tgid, bpf_get_current_uid_gid,
        bpf_ktime_get_ns, bpf_probe_read_kernel,
    },
    macros::{kprobe, kretprobe},
    programs::{ProbeContext, RetProbeContext},
};
use snitchster_common::{ConnectionEvent, SockInfo};

use crate::{EVENTS, TCP_CONNECTING};

// Offsets into struct sock -> __sk_common (struct sock_common)
const SKC_FAMILY_OFF: usize = 16;
const SKC_DADDR_OFF: usize = 0;
const SKC_RCVSADDR_OFF: usize = 4;
const SKC_DPORT_OFF: usize = 12;
const SKC_NUM_OFF: usize = 14;
const SKC_V6_DADDR_OFF: usize = 40;
const SKC_V6_RCVSADDR_OFF: usize = 56;

const AF_INET: u16 = 2;
const AF_INET6: u16 = 10;

// ---- IPv4 ----

#[kprobe]
pub fn kprobe__tcp_v4_connect(ctx: ProbeContext) -> u32 {
    match try_tcp_connect_entry(&ctx) {
        Ok(()) => 0,
        Err(_) => 0,
    }
}

#[kretprobe]
pub fn kretprobe__tcp_v4_connect(ctx: RetProbeContext) -> u32 {
    match try_tcp_connect_return(&ctx) {
        Ok(()) => 0,
        Err(_) => 0,
    }
}

// ---- IPv6 ----

#[kprobe]
pub fn kprobe__tcp_v6_connect(ctx: ProbeContext) -> u32 {
    match try_tcp_connect_entry(&ctx) {
        Ok(()) => 0,
        Err(_) => 0,
    }
}

#[kretprobe]
pub fn kretprobe__tcp_v6_connect(ctx: RetProbeContext) -> u32 {
    match try_tcp_connect_return(&ctx) {
        Ok(()) => 0,
        Err(_) => 0,
    }
}

// ---- Shared logic ----

fn try_tcp_connect_entry(ctx: &ProbeContext) -> Result<(), i64> {
    let sk: u64 = ctx.arg(0).ok_or(1i64)?;
    let pid_tgid = bpf_get_current_pid_tgid();
    let pid = (pid_tgid >> 32) as u32;
    let uid = bpf_get_current_uid_gid() as u32;
    let comm = bpf_get_current_comm().map_err(|_| 1i64)?;
    let ts = unsafe { bpf_ktime_get_ns() };

    let info = SockInfo {
        sock_ptr: sk,
        pid,
        uid,
        comm,
        timestamp_ns: ts,
    };

    TCP_CONNECTING
        .insert(&pid_tgid, &info, 0)
        .map_err(|_| 1i64)?;

    Ok(())
}

fn try_tcp_connect_return(ctx: &RetProbeContext) -> Result<(), i64> {
    let ret: i32 = ctx.ret().ok_or(1i64)?;

    let pid_tgid = bpf_get_current_pid_tgid();

    if ret != 0 {
        let _ = TCP_CONNECTING.remove(&pid_tgid);
        return Ok(());
    }

    let info = unsafe { TCP_CONNECTING.get(&pid_tgid).ok_or(1i64)? };

    let sock_ptr = info.sock_ptr as *const u8;

    // Read address family
    let family: u16 = unsafe {
        bpf_probe_read_kernel(sock_ptr.add(SKC_FAMILY_OFF) as *const u16).map_err(|_| 1i64)?
    };

    let mut event = ConnectionEvent {
        event_type: 1, // TCP
        ip_version: 4,
        protocol: 6, // IPPROTO_TCP
        _pad: 0,
        pid: info.pid,
        uid: info.uid,
        src_addr: [0u8; 16],
        dst_addr: [0u8; 16],
        src_port: 0,
        dst_port: 0,
        comm: info.comm,
        timestamp_ns: info.timestamp_ns,
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
    }

    // Read ports
    let dport_be: u16 = unsafe {
        bpf_probe_read_kernel(sock_ptr.add(SKC_DPORT_OFF) as *const u16).map_err(|_| 1i64)?
    };
    let sport: u16 = unsafe {
        bpf_probe_read_kernel(sock_ptr.add(SKC_NUM_OFF) as *const u16).map_err(|_| 1i64)?
    };

    event.dst_port = u16::from_be(dport_be);
    event.src_port = sport;

    // Submit event to ring buffer
    if let Some(mut buf) = EVENTS.reserve::<ConnectionEvent>(0) {
        unsafe {
            core::ptr::write(buf.as_mut_ptr(), event);
        }
        buf.submit(0);
    }

    // Clean up map entry
    let _ = TCP_CONNECTING.remove(&pid_tgid);

    Ok(())
}
