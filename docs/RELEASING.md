# Releasing BigSnatch

## Prerequisites

- SSH key for AUR: `~/.ssh/id_ed25519_aur` (has a passphrase)
- SSH config (`~/.ssh/config`) maps `aur.archlinux.org` to that key
- GitHub remote uses HTTPS: `https://github.com/invisi101/bigsnatch.git`
- Rust stable + nightly with `rust-src`, `bpf-linker`, `protoc`

## Pre-built binary release

### 1. Build the release package

```bash
cd ~/dev/bigsnatch
./release.sh
```

This builds everything and creates `bigsnatch-<version>-x86_64.tar.gz` in the repo root.

### 2. Tag and push

```bash
git tag v<VERSION>
git push origin main
git push origin v<VERSION>
```

### 3. Create GitHub Release

```bash
gh release create v<VERSION> bigsnatch-<VERSION>-x86_64.tar.gz \
    --title "BigSnatch v<VERSION>" \
    --notes "Pre-built binary for x86_64 Arch Linux.

## Install
\`\`\`bash
tar xzf bigsnatch-<VERSION>-x86_64.tar.gz
cd bigsnatch-<VERSION>-x86_64
sudo ./install.sh
\`\`\`"
```

## AUR release

### 1. Get the SHA256 of the tagged tarball

```bash
curl -sL https://github.com/invisi101/bigsnatch/archive/v<VERSION>.tar.gz | sha256sum
```

### 2. Update the PKGBUILD

Edit `PKGBUILD`:

```
pkgver=<VERSION>
sha256sums=('<THE_NEW_HASH>')
```

### 3. Regenerate .SRCINFO

```bash
makepkg --printsrcinfo > .SRCINFO
```

### 4. Commit and push

```bash
git add PKGBUILD .SRCINFO
git commit -m "Release v<VERSION>: description"
git push origin main
```

### 5. Push to AUR

Run in terminal (needs interactive SSH passphrase):

```bash
eval "$(ssh-agent -s)" && ssh-add ~/.ssh/id_ed25519_aur && rm -rf /tmp/bigsnatch-aur && git clone ssh://aur@aur.archlinux.org/bigsnatch.git /tmp/bigsnatch-aur && cp ~/dev/bigsnatch/PKGBUILD ~/dev/bigsnatch/.SRCINFO /tmp/bigsnatch-aur/ && cd /tmp/bigsnatch-aur && git add PKGBUILD .SRCINFO && git commit -m "Update to v<VERSION>" && git push
```

### 6. Verify

```bash
curl -s "https://aur.archlinux.org/rpc/v5/info?arg[]=bigsnatch" | python3 -c "import sys,json; print(json.load(sys.stdin)['results'][0]['Version'])"
yay -S bigsnatch --rebuild
```
