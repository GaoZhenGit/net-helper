# net-helper 项目说明

## 项目定位
多功能网络诊断调试工具，类似 netcat/nmap 集成体。

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
│   ├── mod.rs        #   公共 API + IsTerminal 检测
│   ├── term.rs       #   交互终端 (crossterm raw mode)
│   └── pipe.rs       #   管道模式 (纯文本 stdin/stdout)
├── net.rs            # 通用网络会话循环 (spawn_receiver + interactive)
├── tls.rs            # TLS 连接 (rustls + webpki-roots)
├── tcp.rs            # TCP 模块（含 -tls 标志）
├── udp.rs            # UDP 模块
├── ws.rs             # WebSocket 模块（-ws，含 WSS + 证书降级）
├── dns.rs            # DNS 模块（含 resolve() 供其他模块复用）
└── version.rs        # 版本号输出

tests/
└── test.py           # 自动测试脚本（管道模式）
```
- `console` 模块通过 `std::io::IsTerminal` 自动检测管道/终端，提供统一的 `poll`/`send`/`recv`/`status` API
- `net.rs` 的 `spawn_receiver` 和 `interactive` 接受任何 `Read + Write`，TCP/TLS/后续模块复用
- `tls.rs` 完全独立，其他模块可直接调用 `TlsStream::connect(sock, domain)`
- `dns.rs` 提供 `resolve(host, port, ipv6)` 统一 DNS 解析，默认 IPv4，`-ipv6` 开启双栈
- `ws.rs` 的 `tls_connect` 内置证书降级逻辑：默认验证 → 失败则重连 + permissive verifier

## CLI 风格
Netcat 风格 flat flags：`-u`/`--udp`、`-t`/`--tcp`、`-tls`、`-ws`/`--ws`、`-d`/`--dns`、`-v`/`--version`、`-h`/`--help`。
全局参数：`-ipv6`/`-6`（IPv6 解析）。main.rs 只做 match 分发，模块内部自行校验参数。

## 构建
```powershell
.\build.ps1              # 增量双平台
.\build.ps1 -Clean       # 清本项目产物后全编（不动依赖）
.\build.ps1 v1.0.0       # 指定版本号
```

## 行为约束
- **禁止自动提交 git**，除非用户明确要求
- **允许自动构建/编译/测试**
- **不引入测试框架**，纯手动 + pipe 测试
- **保持跨平台**，利用 Rust std 内置跨平台能力
- 程序输出使用英文（防止 Windows 终端乱码）
- 会话、文档、注释使用中文
