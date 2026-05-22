# net-helper 项目说明

## 项目定位
多功能网络诊断调试工具，类似 netcat/nmap 集成体。

## 构建环境
- **编译器**: MinGW GCC 12.1.0 (w64devkit)
- **构建系统**: CMake 3.28
- **C++ 标准**: C++17
- **链接方式**: 静态链接（`-static -static-libgcc -static-libstdc++`），产物独立运行，不依赖外部 DLL
- **平台**: 跨平台（Windows/Linux/macOS），平台差异集中在 `src/socket.h`

## 工程结构
```
src/
├── main.cpp           # 入口：flag 解析 → 模块分发
├── socket.h           # 跨平台 Socket RAII 封装 (header-only)
├── net_common.h       # 通用网络工具 (header-only)
└── mod_xxx.h/cpp      # 功能模块，每个模块一个 .h/.cpp 对
```
- 核心层（socket.h, net_common.h）header-only
- 功能模块 .h/.cpp 分离，每模块暴露 `int run_xxx(int argc, char* argv[])` 入口
- CMakeLists.txt 用 `file(GLOB_RECURSE)` 自动收集 src/*.cpp

## CLI 风格
Netcat 风格 flat flags：`-u`/`--udp`、`-t`/`--tcp`、`-l`/`--listen`、`-s`/`--scan` 等。
main.cpp 只做 switch-case 分发，模块内部自行校验参数。

## 行为约束
- **禁止自动提交 git**，除非用户明确要求
- **自动构建/编译**，本项目允许自动构建/编译/测试
- **不引入测试框架**，纯手动测试
- 程序输出使用英文（防止 Windows 终端乱码）
- 会话、文档、注释使用中文
