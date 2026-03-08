#![no_std]
#![no_main]
#![allow(non_snake_case)]

mod tcp;
mod udp;
mod dns;

use aya_ebpf::{
    macros::map,
    maps::{HashMap, PerCpuArray, RingBuf},
};
use snitchster_common::{DnsEvent, SockInfo};

/// Tracks in-flight tcp_connect calls. Key: pid_tgid (u64), Value: SockInfo
#[map]
static TCP_CONNECTING: HashMap<u64, SockInfo> = HashMap::with_max_entries(10240, 0);

/// Ring buffer for connection events → userspace
#[map]
static EVENTS: RingBuf = RingBuf::with_byte_size(256 * 1024, 0);

/// Ring buffer for DNS packet events → userspace
#[map]
static DNS_EVENTS: RingBuf = RingBuf::with_byte_size(256 * 1024, 0);

/// Per-CPU scratch space for building DnsEvent (too large for BPF stack)
#[map]
static DNS_SCRATCH: PerCpuArray<DnsEvent> = PerCpuArray::with_max_entries(1, 0);

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
