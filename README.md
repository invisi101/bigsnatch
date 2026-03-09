# bigsnatch

**Real-time Linux network connection monitor** — an eBPF-powered desktop app that shows you exactly what your computer is reaching out to, which process is doing it, and where it's going.

![BigSnatch](assets/bigsnatch-logo.png)

## Features

| Feature | Description |
|---------|-------------|
| **eBPF kernel probes** | Hooks into `tcp_connect` and `udp_sendmsg` at the kernel level — catches every outgoing connection |
| **Process attribution** | Shows the exact process name, PID, executable path, command line, and user for each connection |
| **DNS correlation** | Captures DNS responses and maps IPs back to domain names in real-time |
| **IPv4 and IPv6** | Full support for both address families |
| **Desktop GUI** | Dark-themed native GUI with sortable connection table, process sidebar, search, and filtering |
| **Auto-launch** | One click from app launcher — daemon starts with a single password prompt and auto-exits when you close the GUI |

## How it works

```
App → OS networking stack → eBPF kprobe → ring buffer → daemon → gRPC → GUI
                ↑
        BigSnatch sees:
        process + IP + port + domain
```

Unlike packet sniffers (Wireshark), BigSnatch sits inside the kernel's networking layer via eBPF. This means it's:

- **Process-aware** — knows exactly which binary made the connection
- **Earlier** — sees the connection before packets leave the machine
- **Cheaper** — no packet copying, just metadata from kernel hooks

## Architecture

```
bigsnatch/
├── snitchster-common/     Shared #[repr(C)] types (kernel ↔ userspace)
├── snitchster-ebpf/       eBPF programs (Rust + aya-rs, compiled to BPF bytecode)
│   ├── tcp.rs             kprobe/kretprobe on tcp_v4_connect, tcp_v6_connect
│   ├── udp.rs             kprobe on udp_sendmsg, udpv6_sendmsg
│   └── dns.rs             Socket filter capturing DNS responses (port 53)
├── snitchster-daemon/     Root daemon (tokio + aya + tonic gRPC)
│   ├── ebpf_loader.rs     Loads and attaches eBPF programs
│   ├── event_processor.rs Polls ring buffers, enriches events with /proc info
│   ├── process_cache.rs   PID → exe path, cmdline, username
│   ├── dns_cache.rs       IP → domain name mapping with TTL
│   ├── dns_parser.rs      DNS wire format parser
│   └── grpc_server.rs     Streams events to GUI over Unix socket
├── snitchster-gui/        Desktop GUI (iced)
│   ├── app.rs             Elm architecture: model/update/view
│   ├── subscription.rs    gRPC stream subscription to daemon
│   └── view/              Toolbar, sidebar, connection table, detail panel, status bar
└── proto/
    └── snitchster.proto   gRPC service definition
```

**All Rust.** eBPF programs, daemon, and GUI share types via a common crate. No C code, no libbpf dependency.

## Install

Requires: Linux kernel >= 5.8 with BTF enabled. Works on Arch, Debian 12+, Ubuntu 22.04+, and other modern distros.

### Option 1: AUR (recommended)

```bash
yay -S bigsnatch-bin
```

Pre-built binary — no compiling, no Rust toolchain needed. Just install and run.

### Option 2: Debian / Ubuntu

Download the latest release from the [Releases](https://github.com/invisi101/bigsnatch/releases) page:

```bash
tar xzf bigsnatch-*-x86_64.tar.gz
cd bigsnatch-*-x86_64
sudo ./install-debian.sh
```

Installs runtime deps (`policykit-1`, Wayland/X11 libs) via apt automatically. Requires kernel >= 5.8 with BTF — Ubuntu 22.04+ and Debian 12+ work out of the box.

To uninstall:

```bash
sudo ./uninstall.sh
```

### Option 3: Manual download (Arch, other distros)

Download the latest release from the [Releases](https://github.com/invisi101/bigsnatch/releases) page:

```bash
tar xzf bigsnatch-*-x86_64.tar.gz
cd bigsnatch-*-x86_64
sudo ./install.sh
```

To uninstall:

```bash
sudo ./uninstall.sh
```

### Option 4: Build from source

```bash
git clone https://github.com/invisi101/bigsnatch.git
cd bigsnatch
./install-deps.sh    # installs build dependencies + Rust toolchain
make all
sudo make install
```

## Usage

### From the app launcher

Search for **BigSnatch** in your application launcher. It will prompt for your password once (to start the eBPF daemon), then the GUI opens. When you close the GUI, the daemon shuts down automatically.

### From the terminal

```bash
# Just the GUI (auto-launches daemon via pkexec)
bigsnatch

# Or manually in two terminals
sudo bigsnatch-daemon   # Terminal 1
bigsnatch               # Terminal 2
```

## GUI

The GUI shows a real-time feed of every outgoing network connection:

- **Toolbar** — search box, protocol filter (All/TCP/UDP), pause, auto-scroll, clear
- **Process sidebar** — lists all processes with connection counts, click to filter
- **Connection table** — sortable columns: Time, Process, PID, Destination, Port, Protocol
- **Detail panel** — full exe path, command line, user, source address for selected connection
- **Status bar** — daemon connection status, total connections, active processes, events/sec

Connections are color-coded:
- **Green** — TCP connections
- **Orange** — UDP connections
- **Purple** — DNS queries (port 53)
- **Blue** — destinations with resolved domain names

## Requirements

- Linux kernel >= 5.8 (RingBuf support, BTF)
- Arch Linux, Debian 12+, Ubuntu 22.04+, or any distro with a modern kernel and BTF enabled
- Root access for eBPF (handled automatically via pkexec)

## Tech stack

| Component | Technology |
|-----------|-----------|
| eBPF programs | Rust + [aya-rs](https://github.com/aya-rs/aya) |
| Daemon | Rust + [tokio](https://tokio.rs) + [aya](https://github.com/aya-rs/aya) |
| IPC | [gRPC](https://grpc.io) over Unix socket ([tonic](https://github.com/hyperium/tonic)) |
| GUI | Rust + [iced](https://github.com/iced-rs/iced) (GPU-accelerated, Elm architecture) |
| DNS parsing | Custom wire-format parser |

## Similar projects

- [Little Snitch](https://www.obdev.at/products/littlesnitch/) (macOS, commercial)
- [OpenSnitch](https://github.com/evilsocket/opensnitch) (Linux, Python/Go)
- [Portmaster](https://github.com/safing/portmaster) (cross-platform, Go)

BigSnatch differs by being all-Rust with eBPF (no kernel module, no C), and focused on monitoring visibility rather than firewall rules.

## License

[GPL-3.0](LICENSE)
