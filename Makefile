.PHONY: all ebpf daemon gui clean install

all: ebpf daemon gui

ebpf:
	cd snitchster-ebpf && cargo +nightly build \
		--target bpfel-unknown-none \
		--release \
		-Z build-std=core

daemon: ebpf
	cargo build --package snitchster-daemon --release

gui:
	cargo build --package snitchster-gui --release

dev-ebpf:
	cd snitchster-ebpf && cargo +nightly build \
		--target bpfel-unknown-none \
		-Z build-std=core

dev-daemon: dev-ebpf
	cargo build --package snitchster-daemon

dev-gui:
	cargo build --package snitchster-gui

dev: dev-ebpf dev-daemon dev-gui

install: all
	install -Dm755 target/release/snitchster-daemon /usr/bin/snitchster-daemon
	install -Dm755 target/release/snitchster-gui /usr/bin/snitchster
	install -Dm644 systemd/snitchster-daemon.service /usr/lib/systemd/system/
	install -Dm644 assets/bigsnatch.svg /usr/share/icons/hicolor/scalable/apps/bigsnatch.svg
	install -Dm644 assets/bigsnatch.desktop /usr/share/applications/bigsnatch.desktop

clean:
	cargo clean
	cd snitchster-ebpf && cargo clean
