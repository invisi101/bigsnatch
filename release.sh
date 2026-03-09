#!/usr/bin/env bash
# release.sh — Build a distributable binary package for BigSnatch
#
# Creates: bigsnatch-<version>-x86_64.tar.gz
# Contains pre-built binaries + install script

set -eo pipefail

BOLD="\e[1m"; GREEN="\e[32m"; RESET="\e[0m"
ok(){ echo -e "${GREEN}✔${RESET} $*"; }

VERSION=$(grep '^version' snitchster-daemon/Cargo.toml | head -1 | sed 's/.*"\(.*\)".*/\1/')
ARCH="x86_64"
PKGNAME="bigsnatch-${VERSION}-${ARCH}"
STAGING="/tmp/${PKGNAME}"

echo -e "${BOLD}Building BigSnatch v${VERSION}...${RESET}\n"

# Build everything
make all
ok "Build complete"

# Stage the package
rm -rf "$STAGING"
mkdir -p "$STAGING"

cp target/release/snitchster-daemon "$STAGING/bigsnatch-daemon"
cp target/release/snitchster-gui "$STAGING/bigsnatch"
cp assets/bigsnatch.svg "$STAGING/"
cp systemd/snitchster-daemon.service "$STAGING/bigsnatch-daemon.service"
cp LICENSE "$STAGING/" 2>/dev/null || true
cp assets/bigsnatch.desktop "$STAGING/"
cp install-debian.sh "$STAGING/"

# Create the install script
cat > "$STAGING/install.sh" <<'INSTALL'
#!/usr/bin/env bash
set -eo pipefail

GREEN="\e[32m"; RED="\e[31m"; BOLD="\e[1m"; RESET="\e[0m"
ok(){ echo -e "${GREEN}✔${RESET} $*"; }
err(){ echo -e "${RED}✘${RESET} $*" >&2; exit 1; }

[[ $EUID -eq 0 ]] || err "Run with sudo: sudo ./install.sh"

DIR="$(cd "$(dirname "$0")" && pwd)"

echo -e "${BOLD}Installing BigSnatch...${RESET}\n"

# Check kernel
KMAJ=$(uname -r | cut -d. -f1)
KMIN=$(uname -r | cut -d. -f2)
if (( KMAJ < 5 || (KMAJ == 5 && KMIN < 8) )); then
    err "Kernel $(uname -r) too old — need >= 5.8 for eBPF RingBuf + BTF"
fi

[[ -f "/sys/kernel/btf/vmlinux" ]] || err "BTF not available — kernel needs CONFIG_DEBUG_INFO_BTF=y"

# Install runtime dependency
pacman -S --needed --noconfirm polkit >/dev/null 2>&1 || true

# Install files
install -Dm755 "$DIR/bigsnatch-daemon" /usr/bin/bigsnatch-daemon
install -Dm755 "$DIR/bigsnatch" /usr/bin/bigsnatch
install -Dm644 "$DIR/bigsnatch-daemon.service" /usr/lib/systemd/system/bigsnatch-daemon.service
install -Dm644 "$DIR/bigsnatch.svg" /usr/share/icons/hicolor/scalable/apps/bigsnatch.svg
install -Dm644 "$DIR/bigsnatch.desktop" /usr/share/applications/bigsnatch.desktop
[[ -f "$DIR/LICENSE" ]] && install -Dm644 "$DIR/LICENSE" /usr/share/licenses/bigsnatch/LICENSE

ok "bigsnatch-daemon  → /usr/bin/bigsnatch-daemon"
ok "bigsnatch (GUI)   → /usr/bin/bigsnatch"
ok "systemd service   → /usr/lib/systemd/system/bigsnatch-daemon.service"
ok "desktop entry     → /usr/share/applications/bigsnatch.desktop"
ok "icon              → /usr/share/icons/hicolor/scalable/apps/bigsnatch.svg"

# Update icon cache
gtk-update-icon-cache -f /usr/share/icons/hicolor/ 2>/dev/null || true

echo -e "\n${BOLD}Done.${RESET} Launch BigSnatch from your app launcher or run: bigsnatch"
INSTALL
chmod +x "$STAGING/install.sh"

# Create uninstall script
cat > "$STAGING/uninstall.sh" <<'UNINSTALL'
#!/usr/bin/env bash
set -eo pipefail

GREEN="\e[32m"; RED="\e[31m"; BOLD="\e[1m"; RESET="\e[0m"
ok(){ echo -e "${GREEN}✔${RESET} Removed $*"; }

[[ $EUID -eq 0 ]] || { echo -e "${RED}✘${RESET} Run with sudo: sudo ./uninstall.sh" >&2; exit 1; }

echo -e "${BOLD}Uninstalling BigSnatch...${RESET}\n"

systemctl stop bigsnatch-daemon 2>/dev/null || true
systemctl disable bigsnatch-daemon 2>/dev/null || true

for f in \
    /usr/bin/bigsnatch-daemon \
    /usr/bin/bigsnatch \
    /usr/lib/systemd/system/bigsnatch-daemon.service \
    /usr/share/icons/hicolor/scalable/apps/bigsnatch.svg \
    /usr/share/applications/bigsnatch.desktop \
    /usr/share/licenses/bigsnatch/LICENSE; do
    [[ -f "$f" ]] && rm -f "$f" && ok "$f"
done

rmdir /usr/share/licenses/bigsnatch 2>/dev/null || true
gtk-update-icon-cache -f /usr/share/icons/hicolor/ 2>/dev/null || true

echo -e "\n${BOLD}BigSnatch uninstalled.${RESET}"
UNINSTALL
chmod +x "$STAGING/uninstall.sh"

ok "Package staged at $STAGING"

# Create tarball
cd /tmp
tar czf "${PKGNAME}.tar.gz" "$PKGNAME"
mv "${PKGNAME}.tar.gz" "$OLDPWD/"
ok "Created ${PKGNAME}.tar.gz"

echo -e "\nUpload to GitHub Releases, users install with:"
echo "  tar xzf ${PKGNAME}.tar.gz"
echo "  cd ${PKGNAME}"
echo "  sudo ./install.sh"
