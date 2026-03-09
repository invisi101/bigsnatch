#!/usr/bin/env bash
# release-debian.sh — Build a Debian/Ubuntu-compatible binary package for BigSnatch
#
# Uses cargo-zigbuild to target glibc 2.31 (Debian 11+ / Ubuntu 20.04+)
# Creates: bigsnatch-<version>-x86_64-debian.tar.gz

set -eo pipefail

BOLD="\e[1m"; GREEN="\e[32m"; RED="\e[31m"; RESET="\e[0m"
ok(){ echo -e "${GREEN}✔${RESET} $*"; }
err(){ echo -e "${RED}✘${RESET} $*" >&2; exit 1; }

command -v zig >/dev/null 2>&1 || err "zig not found — install with: sudo pacman -S zig"
command -v cargo-zigbuild >/dev/null 2>&1 || err "cargo-zigbuild not found — install with: cargo install cargo-zigbuild"

VERSION=$(grep '^version' snitchster-daemon/Cargo.toml | head -1 | sed 's/.*"\(.*\)".*/\1/')
ARCH="x86_64"
TARGET="x86_64-unknown-linux-gnu.2.31"
PKGNAME="bigsnatch-${VERSION}-${ARCH}-debian"
STAGING="/tmp/${PKGNAME}"

echo -e "${BOLD}Building BigSnatch v${VERSION} for Debian/Ubuntu (glibc >= 2.31)...${RESET}\n"

# eBPF is portable bytecode — same build as Arch
cd snitchster-ebpf && cargo +nightly build \
    --target bpfel-unknown-none \
    --release \
    -Z build-std=core && cd ..
ok "eBPF build complete"

# Daemon and GUI targeting older glibc
cargo zigbuild --package snitchster-daemon --release --target "$TARGET"
ok "Daemon build complete"

cargo zigbuild --package snitchster-gui --release --target "$TARGET"
ok "GUI build complete"

# Stage the package
rm -rf "$STAGING"
mkdir -p "$STAGING"

cp "target/x86_64-unknown-linux-gnu/release/snitchster-daemon" "$STAGING/bigsnatch-daemon"
cp "target/x86_64-unknown-linux-gnu/release/snitchster-gui" "$STAGING/bigsnatch"
cp assets/bigsnatch.svg "$STAGING/"
cp systemd/snitchster-daemon.service "$STAGING/bigsnatch-daemon.service"
cp LICENSE "$STAGING/" 2>/dev/null || true
cp assets/bigsnatch.desktop "$STAGING/"
cp install-debian.sh "$STAGING/"
cp release.sh "$STAGING/" 2>/dev/null || true

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
    /usr/share/doc/bigsnatch/LICENSE; do
    [[ -f "$f" ]] && rm -f "$f" && ok "$f"
done

rmdir /usr/share/doc/bigsnatch 2>/dev/null || true
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

echo -e "\nUpload to GitHub Releases:"
echo "  gh release upload v${VERSION} ${PKGNAME}.tar.gz"
echo ""
echo "Users install with:"
echo "  tar xzf ${PKGNAME}.tar.gz"
echo "  cd ${PKGNAME}"
echo "  sudo ./install-debian.sh"
