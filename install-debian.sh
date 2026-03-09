#!/usr/bin/env bash
# install-debian.sh — Install BigSnatch pre-built binaries on Debian/Ubuntu
#
# Usage: Download the release tarball, extract, then run:
#   tar xzf bigsnatch-*-x86_64.tar.gz
#   cd bigsnatch-*-x86_64
#   sudo ../install-debian.sh
#
# Or if included in the tarball:
#   sudo ./install-debian.sh

set -eo pipefail

GREEN="\e[32m"; RED="\e[31m"; BOLD="\e[1m"; RESET="\e[0m"
ok(){ echo -e "${GREEN}✔${RESET} $*"; }
err(){ echo -e "${RED}✘${RESET} $*" >&2; exit 1; }

[[ $EUID -eq 0 ]] || err "Run with sudo: sudo ./install-debian.sh"

DIR="$(cd "$(dirname "$0")" && pwd)"

echo -e "${BOLD}Installing BigSnatch on Debian/Ubuntu...${RESET}\n"

# ---------- Kernel checks ----------
KMAJ=$(uname -r | cut -d. -f1)
KMIN=$(uname -r | cut -d. -f2)
if (( KMAJ < 5 || (KMAJ == 5 && KMIN < 8) )); then
    err "Kernel $(uname -r) too old — need >= 5.8 for eBPF RingBuf + BTF"
fi

if [[ ! -f "/sys/kernel/btf/vmlinux" ]]; then
    err "BTF not available — kernel needs CONFIG_DEBUG_INFO_BTF=y
    On Ubuntu/Debian, try: sudo apt install linux-image-$(uname -r) linux-tools-$(uname -r)"
fi

# ---------- Runtime dependencies ----------
echo -e "${BOLD}Installing runtime dependencies...${RESET}"
apt-get update -qq 2>&1 || true
apt-get install -y pkexec libwayland-client0 libxcb1 libfontconfig1 libfreetype6 2>&1 || \
    apt-get install -y policykit-1 libwayland-client0 libxcb1 libfontconfig1 libfreetype6 2>&1 || \
    echo -e "  (some packages may not be available — install polkit/pkexec manually if needed)"
ok "Runtime dependencies installed"

# ---------- Install files ----------
echo -e "\n${BOLD}Installing binaries and config...${RESET}"

install -Dm755 "$DIR/bigsnatch-daemon" /usr/bin/bigsnatch-daemon
install -Dm755 "$DIR/bigsnatch" /usr/bin/bigsnatch
install -Dm644 "$DIR/bigsnatch-daemon.service" /usr/lib/systemd/system/bigsnatch-daemon.service
install -Dm644 "$DIR/bigsnatch.svg" /usr/share/icons/hicolor/scalable/apps/bigsnatch.svg
install -Dm644 "$DIR/bigsnatch.desktop" /usr/share/applications/bigsnatch.desktop
[[ -f "$DIR/LICENSE" ]] && install -Dm644 "$DIR/LICENSE" /usr/share/doc/bigsnatch/LICENSE

ok "bigsnatch-daemon  → /usr/bin/bigsnatch-daemon"
ok "bigsnatch (GUI)   → /usr/bin/bigsnatch"
ok "systemd service   → /usr/lib/systemd/system/bigsnatch-daemon.service"
ok "desktop entry     → /usr/share/applications/bigsnatch.desktop"
ok "icon              → /usr/share/icons/hicolor/scalable/apps/bigsnatch.svg"

# Update icon cache
gtk-update-icon-cache -f /usr/share/icons/hicolor/ 2>/dev/null || true

# Reload systemd
systemctl daemon-reload 2>/dev/null || true

echo -e "\n${BOLD}Done.${RESET} Launch BigSnatch from your app launcher or run: bigsnatch"
