#!/usr/bin/env bash
# install-deps.sh — Install BigSnatch build and runtime dependencies (Arch Linux)

set -eo pipefail

GREEN="\e[32m"; RED="\e[31m"; BOLD="\e[1m"; RESET="\e[0m"
ok(){ echo -e "${GREEN}✔${RESET} $*"; }
err(){ echo -e "${RED}✘${RESET} $*" >&2; }

# ---------- System packages ----------
echo -e "\n${BOLD}Installing system packages...${RESET}"

PACKAGES=(
    # Build
    base-devel
    protobuf          # protoc compiler for gRPC
    # GUI (iced framework)
    wayland
    libxcb
    fontconfig
    freetype2
    # Runtime
    polkit            # pkexec for privilege escalation
)

sudo pacman -S --needed --noconfirm "${PACKAGES[@]}"
ok "System packages installed"

# ---------- Rust toolchain ----------
echo -e "\n${BOLD}Setting up Rust toolchain...${RESET}"

if ! command -v rustup >/dev/null 2>&1; then
    err "rustup not found — install from https://rustup.rs"
    exit 1
fi

rustup toolchain install nightly --component rust-src
ok "Nightly toolchain with rust-src"

rustup toolchain install stable
ok "Stable toolchain"

# ---------- bpf-linker ----------
echo -e "\n${BOLD}Installing bpf-linker...${RESET}"

if command -v bpf-linker >/dev/null 2>&1; then
    ok "bpf-linker already installed"
else
    cargo install bpf-linker
    ok "bpf-linker installed"
fi

# ---------- Kernel check ----------
echo -e "\n${BOLD}Checking kernel...${RESET}"

KVER=$(uname -r | cut -d. -f1-2)
KMAJ=$(echo "$KVER" | cut -d. -f1)
KMIN=$(echo "$KVER" | cut -d. -f2)

if (( KMAJ > 5 || (KMAJ == 5 && KMIN >= 8) )); then
    ok "Kernel $KVER (>= 5.8 required for eBPF RingBuf + BTF)"
else
    err "Kernel $KVER is too old — need >= 5.8 for eBPF RingBuf and BTF support"
    exit 1
fi

if [[ -f "/sys/kernel/btf/vmlinux" ]]; then
    ok "BTF enabled"
else
    err "BTF not available at /sys/kernel/btf/vmlinux — kernel may need CONFIG_DEBUG_INFO_BTF=y"
    exit 1
fi

# ---------- Done ----------
echo -e "\n${BOLD}All dependencies installed.${RESET} Build with:"
echo "  make all"
echo "  sudo make install"
