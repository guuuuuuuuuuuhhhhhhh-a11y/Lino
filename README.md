# Linux Distro Manager (CLI + Core)

This repository contains the initial CLI and core library for managing minimal CLI environments on Windows (via WSL or Docker) with support for 10+ Linux distributions.

## Build

```bash
cargo build
```

## Run

```bash
cargo run -p ldm-cli -- list-distros --registry distributions.json
cargo run -p ldm-cli -- versions ubuntu --registry distributions.json
```

## Supported distributions (initial)
Ubuntu, Debian, Fedora, Alpine, Arch Linux, Kali, ParrotOS, openSUSE, Rocky Linux, AlmaLinux, Oracle Linux, Gentoo, Void Linux

GUI (Tauri) will be added in the next iteration using the same core.