# net-helper 项目说明

## 项目定位
多功能网络诊断调试工具，类似 netcat/nmap 集成体。

## 技术栈
- **语言**: Rust (edition 2021)
- **构建系统**: Cargo
- **Windows 链接器**: MinGW GCC 12.1.0 (w64devkit) / rust-lld
- **Linux 交叉编译**: rust-lld + musl target
- **平台**: 跨平台（Windows/Linux/macOS），零 `#[cfg]` 条件编译

## 工程结构
```
src/
├── main.rs      # 入口：flag 解析 → 模块分发 + 全局 stdout 锁
├── udp.rs       # UDP 模块
├── tcp.rs       # TCP 模块
├── dns.rs       # DNS 模块
└── version.rs   # 版本输出

build.rs          # 构建时生成版本号（时间戳 / 环境变量覆盖）
build.ps1         # 一键双平台构建脚本
.cargo/config.toml # musl target 链接器配置
```
- `main.rs` 中的 `OUT` (Mutex) 和 `put()` 供各模块共享，序列化多线程控制台输出
- 每模块暴露 `pub fn run(args: &[String]) -> i32` 入口

## CLI 风格
Netcat 风格 flat flags：`-u`/`--udp`、`-t`/`--tcp`、`-d`/`--dns`、`-v`/`--version` 等。
main.rs 只做 match 分发，模块内部自行校验参数。

## 构建
```powershell
.\build.ps1              # 双平台（Windows + Linux musl）
.\build.ps1 v1.0.0       # 指定版本号
```

## 行为约束
- **禁止自动提交 git**，除非用户明确要求
- **允许自动构建/编译/测试**
- **不引入测试框架**，纯手动测试
- **保持跨平台**，利用 Rust std 内置跨平台能力
- 程序输出使用英文（防止 Windows 终端乱码）
- 会话、文档、注释使用中文
