# net-helper 工程文档

## 技术栈

- **语言**: Rust (edition 2021)
- **构建系统**: Cargo
- **Windows 链接器**: MinGW GCC 12.1.0 (w64devkit) / rust-lld
- **Linux 交叉编译**: cargo-zigbuild + musl target
- **平台**: 跨平台（Windows/Linux/macOS），零 `#[cfg]` 条件编译

## 工程结构

```
src/
├── main.rs           # 入口：flag 解析 → 模块分发
├── console/          # 终端 I/O（自动切换交互/管道模式）
│   ├── mod.rs        #   公共 API + IsTerminal 检测 + eol() 行尾自适应
│   ├── term.rs       #   交互终端 (crossterm raw mode)
│   └── pipe.rs       #   管道模式 (纯文本 stdin/stdout)
├── net.rs            # 通用网络会话 (spawn_receiver + interactive + connect_timeout)
├── tls.rs            # TLS 连接 (rustls + webpki-roots)
├── tcp.rs            # TCP 模块（含 -tls 标志）
├── udp.rs            # UDP 模块
├── ws.rs             # WebSocket 模块（-ws，含 WSS + 证书降级）
├── dns.rs            # DNS 模块（含 resolve() 供其他模块复用）
└── version.rs        # 版本号输出

tests/
├── test.py           # 自动测试脚本（12 项，管道模式）
├── ws_echo.py        # 本地 WS/WSS echo server（测试用）
├── cert.pem          # 自签证书
└── key.pem           # 自签密钥
```

### 模块要点

- `console` 模块通过 `std::io::IsTerminal` 自动检测管道/终端，提供统一的 `poll`/`send`/`recv`/`recv_from`/`status`/`println` API。`eol()` 函数按后端类型返回 `\r\n`（终端）或 `\n`（管道），确保 raw mode 光标归零列同时 pipe 输出清洁。
- `net.rs` 的 `spawn_receiver` 和 `interactive` 接受任何 `Read + Write`，TCP/TLS/后续模块复用。`connect_timeout` 通过 channel + thread 实现连接超时。
- `tls.rs` 完全独立，其他模块可直接调用 `TlsStream::connect(sock, domain)`。`Read` 实现吞掉 `WouldBlock` 后仍读内部缓冲区，防止 flush 路径已读入的数据丢失。
- `dns.rs` 提供 `resolve(host, port, ipv6)` 统一 DNS 解析。默认 IPv4 过滤，`-ipv6` 开启双栈。host 为 IP 字面量时跳过过滤。
- `ws.rs` 的 `tls_connect` 内置证书降级逻辑：默认 webpki-roots 验证 → 失败则重连 + permissive verifier（验证签名、不验证证书链）。

## 构建

### 前置条件

```powershell
rustup target add x86_64-unknown-linux-musl aarch64-unknown-linux-musl
cargo install cargo-zigbuild
```

### 构建命令

```powershell
.\build.ps1              # 增量三平台（Windows / Linux x86_64 / Linux ARM64）
.\build.ps1 -Clean       # 清本项目产物后全编（不动依赖）
.\build.ps1 1.0.0        # 指定发布版本号（semver 格式，可选 v 前缀）
```

### 产物

| 文件 | 目标平台 |
|------|---------|
| `target/net-helper.exe` | Windows x86_64 (MinGW, static) |
| `target/net-helper` | Linux x86_64 (musl, static) |
| `target/net-helper-arm64` | Linux ARM64 (musl, static) |

## 版本号规则

- 三段式版本号在 `Cargo.toml` 的 `version` 字段配置（当前：`1.0.0`）
- 默认版本格式：`{Cargo.toml version}+{YYYYMMDD.HHmm}`，如 `1.0.0+20260527.0944`（符合 semver build metadata 规范）
- 通过 `build.ps1 1.0.0` 传参可覆盖版本号，跳过时间戳
- 版本号通过 cargo 环境变量 `NETHELPER_VERSION` 注入，由 `version.rs` 输出

## 测试

```powershell
pip install websockets    # WS/WSS 本地测试依赖
python tests/test.py
```

12 项测试：version、help、unknown flag、DNS qq.com/no args、UDP、TCP HTTP/EOF、TCP TLS、WS local、WSS local、WSS public。

版本号测试使用正则 `^\d+\.\d+\.\d+\+\d{8}\.\d{4}$` 校验格式，不绑定具体数值。
