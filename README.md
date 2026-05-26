# net-helper

A multi-function network diagnostic tool — portable, static, zero runtime dependencies.

## What it does

- **UDP** — send messages, receive replies in real time
- **TCP** — full-duplex interactive session, like telnet
- **TCP + TLS** — encrypted connect with automatic system CA trust
- **DNS** — resolve domains, showing all IPv4/IPv6 records

Supports both interactive terminal mode and pipe/script mode.

## Quick start

```bash
# UDP
net-helper -u example.com 2077

# TCP (HTTP)
net-helper -t www.baidu.com 80

# TCP with TLS (HTTPS)
net-helper -t -tls www.baidu.com 443

# DNS
net-helper -d qq.com

# Help
net-helper -h
```

Type and press Enter to send. `/quit` to exit. In pipes, close stdin to exit.

## Build

**Prerequisites:** [Rust](https://rustup.rs) (stable), [w64devkit](https://github.com/skeeto/w64devkit), [cargo-zigbuild](https://github.com/rust-cross/cargo-zigbuild), musl target.

```powershell
# One-time setup
rustup target add x86_64-unknown-linux-musl
cargo install cargo-zigbuild

# Build both platforms
.\build.ps1

# Clean rebuild (project only, deps kept)
.\build.ps1 -Clean

# Pin version
.\build.ps1 v1.0.0
```

Output: `target/net-helper.exe` (Windows) and `target/net-helper` (Linux musl, static).

## Test

```powershell
python tests/test.py
```

## Usage

```
Usage:
  net-helper -u  <ip|domain> <port>   UDP send/receive
  net-helper -t  <ip|domain> <port>   TCP connect
  net-helper -t  -tls <ip|domain> <port>  TCP with TLS
  net-helper -d  <domain>             DNS lookup
  net-helper -v, --version            Show version
  net-helper -h, --help               Show this help
```

## Platform support

| Platform | Binary | Notes |
|---|---|---|
| Windows | `net-helper.exe` | MinGW, static |
| Linux | `net-helper` | musl, static-pie |
