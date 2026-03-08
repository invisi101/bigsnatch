use anyhow::{Context, Result};
use aya::{
    include_bytes_aligned,
    programs::{KProbe, SocketFilter},
    Ebpf,
};
use tracing::info;

/// Load and attach all eBPF programs.
pub fn load_and_attach() -> Result<Ebpf> {
    #[cfg(debug_assertions)]
    let mut ebpf = Ebpf::load(include_bytes_aligned!(
        "../../snitchster-ebpf/target/bpfel-unknown-none/debug/snitchster-ebpf"
    ))?;

    #[cfg(not(debug_assertions))]
    let mut ebpf = Ebpf::load(include_bytes_aligned!(
        "../../snitchster-ebpf/target/bpfel-unknown-none/release/snitchster-ebpf"
    ))?;

    // TCP IPv4
    let prog: &mut KProbe = ebpf
        .program_mut("kprobe__tcp_v4_connect")
        .context("kprobe__tcp_v4_connect not found")?
        .try_into()?;
    prog.load()?;
    prog.attach("tcp_v4_connect", 0)?;
    info!("Attached kprobe: tcp_v4_connect");

    let prog: &mut KProbe = ebpf
        .program_mut("kretprobe__tcp_v4_connect")
        .context("kretprobe__tcp_v4_connect not found")?
        .try_into()?;
    prog.load()?;
    prog.attach("tcp_v4_connect", 0)?;
    info!("Attached kretprobe: tcp_v4_connect");

    // TCP IPv6
    let prog: &mut KProbe = ebpf
        .program_mut("kprobe__tcp_v6_connect")
        .context("kprobe__tcp_v6_connect not found")?
        .try_into()?;
    prog.load()?;
    prog.attach("tcp_v6_connect", 0)?;
    info!("Attached kprobe: tcp_v6_connect");

    let prog: &mut KProbe = ebpf
        .program_mut("kretprobe__tcp_v6_connect")
        .context("kretprobe__tcp_v6_connect not found")?
        .try_into()?;
    prog.load()?;
    prog.attach("tcp_v6_connect", 0)?;
    info!("Attached kretprobe: tcp_v6_connect");

    // UDP IPv4
    let prog: &mut KProbe = ebpf
        .program_mut("kprobe__udp_sendmsg")
        .context("kprobe__udp_sendmsg not found")?
        .try_into()?;
    prog.load()?;
    prog.attach("udp_sendmsg", 0)?;
    info!("Attached kprobe: udp_sendmsg");

    // UDP IPv6
    let prog: &mut KProbe = ebpf
        .program_mut("kprobe__udpv6_sendmsg")
        .context("kprobe__udpv6_sendmsg not found")?
        .try_into()?;
    prog.load()?;
    prog.attach("udpv6_sendmsg", 0)?;
    info!("Attached kprobe: udpv6_sendmsg");

    // DNS socket filter — attach to a raw socket to capture DNS responses
    match attach_dns_filter(&mut ebpf) {
        Ok(()) => info!("Attached socket filter: dns_filter"),
        Err(e) => {
            tracing::warn!("Failed to attach DNS socket filter (domains won't resolve): {:#}", e);
        }
    }

    Ok(ebpf)
}

fn attach_dns_filter(ebpf: &mut Ebpf) -> Result<()> {
    let prog: &mut SocketFilter = ebpf
        .program_mut("dns_filter")
        .context("dns_filter not found")?
        .try_into()?;
    prog.load()?;

    let sock = socket_with_filter(prog)?;
    // Leak the socket fd so it stays open for the lifetime of the daemon
    std::mem::forget(sock);
    Ok(())
}

/// Create a raw socket and attach a BPF socket filter to it.
fn socket_with_filter(prog: &mut SocketFilter) -> Result<std::os::unix::io::OwnedFd> {
    use std::os::fd::{AsFd, FromRawFd, OwnedFd};

    // Create AF_PACKET raw socket to see all packets
    let fd = unsafe {
        libc::socket(
            libc::AF_PACKET,
            libc::SOCK_RAW | libc::SOCK_NONBLOCK | libc::SOCK_CLOEXEC,
            (libc::ETH_P_ALL as u16).to_be() as i32,
        )
    };
    if fd < 0 {
        return Err(anyhow::anyhow!(
            "Failed to create raw socket: {}",
            std::io::Error::last_os_error()
        ));
    }

    let owned = unsafe { OwnedFd::from_raw_fd(fd) };

    prog.attach(owned.as_fd())
        .context("Failed to attach dns_filter to raw socket")?;

    Ok(owned)
}
