# net-helper 项目说明

## 项目定位
多功能网络诊断调试工具，类似 netcat/nmap 集成体。

## 技术栈
- **语言**: Rust (edition 2021)
- **构建系统**: Cargo
- **平台**: 跨平台（Windows/Linux/macOS），零 `#[cfg]` 条件编译

详细工程结构、构建命令、版本号规则见 [docs/project.md](docs/project.md)。

## CLI 风格
Netcat 风格 flat flags：`-u`/`--udp`、`-t`/`--tcp`、`-tls`、`-ws`/`--ws`、`-d`/`--dns`、`-v`/`--version`、`-h`/`--help`。
全局参数：`-ipv6`/`-6`（IPv6 解析）。main.rs 只做 match 分发，模块内部自行校验参数。

## 行为约束
- **禁止自动提交 git**，除非用户明确要求
- **允许自动构建/编译/测试**
- **不引入测试框架**，纯手动 + pipe 测试
- **保持跨平台**，利用 Rust std 内置跨平台能力
- 程序输出使用英文（防止 Windows 终端乱码）
- 会话、文档、注释使用中文
