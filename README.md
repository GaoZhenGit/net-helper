# net-helper

A multi-function network diagnostic tool, like a portable cross-platform nc/nslookup combo.

## What it does

- **UDP send/receive** — connect to any UDP server, send messages, see replies in real time. Supports both IP and domain name.
- **DNS lookup** — query system DNS to see all resolved addresses with IPv4/IPv6 labels.

## Quick start

Get the binary for your platform from the release page, or build it yourself (see below).

```bash
# UDP mode — connect and chat
net-helper -u example.com 2077

# DNS lookup
net-helper -d baidu.com

# Show version
net-helper --version
```

In UDP mode, type your message and press Enter to send. Type `/quit` to exit.

## Build from source

**Prerequisites:**
- Windows with [w64devkit](https://github.com/skeeto/w64devkit) (MinGW GCC, CMake, make)
- [Zig](https://ziglang.org/download) 0.14+ (for Linux cross-compilation)

**One command:**

```powershell
.\build.ps1
```

This produces:
- `build-win\net-helper.exe` — Windows
- `build-linux\net-helper` — Linux (musl, static)

To pin a specific version:

```powershell
.\build.ps1 v2026.12.25.1800
```

## Platform support

| Platform | Binary | Notes |
|---|---|---|
| Windows | `net-helper.exe` | Static link, no DLLs needed |
| Linux | `net-helper` | musl libc, static link, runs on any kernel 2.6+ |

## Usage reference

```
Usage:
  net-helper -u <ip|domain> <port>   UDP send/receive
  net-helper -d <domain>              DNS lookup
  net-helper -v, --version            Show version
```
