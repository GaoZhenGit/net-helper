# net-helper

A multi-function network diagnostic tool — portable, static, zero-dependency.

## What it does

- **UDP send/receive** — connect to any UDP server, send messages, see replies in real time
- **TCP connect** — interact with TCP servers (HTTP, SMTP, telnet, etc.), full-duplex I/O
- **DNS lookup** — resolve domains via system DNS, see all IPv4/IPv6 records

## Quick start

```bash
# UDP chat
net-helper -u example.com 2077

# TCP — talk HTTP
net-helper -t www.baidu.com 80

# DNS lookup
net-helper -d baidu.com

# Show version
net-helper --version
```

In UDP/TCP mode, type your message and press Enter to send. Type `/quit` to exit.

## Build from source

**Prerequisites:**
- [Rust](https://rustup.rs) (stable)
- [w64devkit](https://github.com/skeeto/w64devkit) (MinGW, for Windows target)
- musl target: `rustup target add x86_64-unknown-linux-musl`

**One command:**

```powershell
.\build.ps1
```

This produces:
- `build-win\net-helper.exe` — Windows (1.3 MB)
- `build-linux\net-helper` — Linux musl static (743 KB)

To pin a specific version:

```powershell
.\build.ps1 v2026.12.25.1800
```

## Platform support

| Platform | Binary | Notes |
|---|---|---|
| Windows | `net-helper.exe` | MinGW, static link |
| Linux | `net-helper` | musl libc, static-pie, runs on any kernel |

## Usage

```
Usage:
  net-helper -u <ip|domain> <port>   UDP send/receive
  net-helper -t <ip|domain> <port>   TCP connect
  net-helper -d <domain>              DNS lookup
  net-helper -v, --version            Show version
```
