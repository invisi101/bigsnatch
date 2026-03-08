use anyhow::{Context, Result};
use aya::{
    include_bytes_aligned,
    programs::KProbe,
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

    Ok(ebpf)
}
