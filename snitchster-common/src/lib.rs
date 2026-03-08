#![no_std]

/// Connection event emitted from eBPF to userspace via RingBuf.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ConnectionEvent {
    pub event_type: u8,       // 1=TCP, 2=UDP
    pub ip_version: u8,       // 4 or 6
    pub protocol: u8,         // 6=TCP, 17=UDP
    pub _pad: u8,
    pub pid: u32,
    pub uid: u32,
    pub src_addr: [u8; 16],
    pub dst_addr: [u8; 16],
    pub src_port: u16,
    pub dst_port: u16,
    pub comm: [u8; 16],
    pub timestamp_ns: u64,
}

/// DNS packet event passed to userspace for parsing
#[repr(C)]
#[derive(Clone, Copy)]
pub struct DnsEvent {
    pub pid: u32,
    pub len: u32,
    pub data: [u8; 512],
    pub timestamp_ns: u64,
}

/// Tracks in-flight tcp_connect between kprobe and kretprobe.
/// Keyed by pid_tgid in the BPF HashMap.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SockInfo {
    pub sock_ptr: u64,
    pub pid: u32,
    pub uid: u32,
    pub comm: [u8; 16],
    pub timestamp_ns: u64,
}

#[cfg(feature = "userspace")]
unsafe impl Send for ConnectionEvent {}
#[cfg(feature = "userspace")]
unsafe impl Sync for ConnectionEvent {}
#[cfg(feature = "userspace")]
unsafe impl Send for DnsEvent {}
#[cfg(feature = "userspace")]
unsafe impl Sync for DnsEvent {}
