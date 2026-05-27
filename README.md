# net-helper

A multi-function network diagnostic tool — portable, static, zero runtime dependencies.

## What it does

- **UDP** — send messages, receive replies in real time
- **TCP** — full-duplex interactive session, like telnet
- **TCP + TLS** — encrypted connect with automatic system CA trust
- **WebSocket / WSS** — with automatic certificate verification fallback (strict → permissive)
- **DNS** — resolve domains, showing all IPv4/IPv6 records

Supports both interactive terminal mode and pipe/script mode.

## Quick start

```bash
# UDP
net-helper -u example.com 2077

# TCP (HTTP)
net-helper -t example.com 80

# TCP with TLS (HTTPS)
net-helper -t -tls www.baidu.com 443

# WebSocket / WSS
net-helper -ws ws://127.0.0.1:9001/echo
net-helper -ws wss://127.0.0.1:9002/echo

# DNS
net-helper -d qq.com

# IPv6双栈
net-helper -t -ipv6 example.com 80

# Help / Version
net-helper -h
net-helper -v
```

Type and press Enter to send. `/quit` to exit. In pipes, close stdin to exit.

## Build

**Prerequisites:** [Rust](https://rustup.rs) (stable), [w64devkit](https://github.com/skeeto/w64devkit), [cargo-zigbuild](https://github.com/rust-cross/cargo-zigbuild).

```powershell
# One-time setup
rustup target add x86_64-unknown-linux-musl aarch64-unknown-linux-musl
cargo install cargo-zigbuild

# Build all platforms
.\build.ps1

# Clean rebuild (project only, deps kept)
.\build.ps1 -Clean

# Pin version
.\build.ps1 1.0.0
```

Output:

| File | Target |
|------|--------|
| `target/net-helper.exe` | Windows x86_64 (MinGW, static) |
| `target/net-helper` | Linux x86_64 (musl, static) |
| `target/net-helper-arm64` | Linux ARM64 (musl, static) |

## Version

- 三段式版本号在 `Cargo.toml` 的 `version` 字段配置
- 默认格式：`{version}+{timestamp}`，如 `1.0.0+20260527.0944`
- 通过 `build.ps1 1.0.0` 覆盖为纯净版本号

## Test

```powershell
pip install websockets    # WS/WSS 本地测试依赖
python tests/test.py
```

12 tests: version, help, unknown flag, DNS, UDP, TCP HTTP/EOF, TCP TLS, WS local, WSS local, WSS public.

## Usage

```
Usage:
  net-helper -u  <ip|domain> <port>   UDP send/receive
  net-helper -t  <ip|domain> <port>   TCP connect
  net-helper -t  -tls <ip|domain> <port>  TCP with TLS
  net-helper -ws <ws://host[:port][/path]>   WebSocket
  net-helper -d  <domain>             DNS lookup
  net-helper -v, --version            Show version
  net-helper -h, --help               Show this help

Global: -ipv6 / -6  enable IPv6 dual-stack resolution
```

## Platform support

| Platform | Binary | Notes |
|---|---|---|
| Windows | `net-helper.exe` | MinGW, static |
| Linux x86_64 | `net-helper` | musl, static |
| Linux ARM64 | `net-helper-arm64` | musl, static |
