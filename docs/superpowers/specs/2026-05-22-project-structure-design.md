# net-helper 工程结构设计

## 概述

net-helper 是一个多功能网络诊断调试工具，类似 netcat/nmap 的集成体。
采用 netcat 风格 flat flags (`-u`, `-t`, `-s` 等)，跨平台（Windows/Linux/macOS），C++17 + CMake + MinGW。

## 目录结构

```
net-helper/
├── CMakeLists.txt
├── src/
│   ├── main.cpp              # 入口：flag 解析 → 模块分发
│   ├── socket.h               # 跨平台 socket RAII 封装 (header-only)
│   ├── net_common.h           # 通用网络工具函数 (header-only)
│   ├── mod_udp.h / mod_udp.cpp
│   ├── mod_tcp.h / mod_tcp.cpp
│   ├── mod_scan.h / mod_scan.cpp
│   ├── mod_dns.h / mod_dns.cpp
│   └── ...                    # 后续功能模块
└── build/                     # out-of-source build (gitignore)
```

## 分层架构

```
┌──────────────────────────────────┐
│  main.cpp     flag 解析 & 分发    │
├──────────────────────────────────┤
│  mod_*.cpp    功能模块 (入口函数)  │
│  udp / tcp / scan / dns / http   │
├──────────────────────────────────┤
│  socket.h    跨平台 Socket RAII   │
│  net_common.h  地址解析/工具      │
└──────────────────────────────────┘
```

## 核心层

### socket.h (header-only)

基类 `Socket` 管理 fd 生命周期（RAII），派生类覆盖不同协议：

| 类 | 职责 | 使用者 |
|---|---|---|
| `Socket` | fd RAII 基类，move-only | — |
| `UdpSocket` | socket() + bind(0) + send/recv | mod_udp |
| `TcpSocket` | socket() + connect() + send/recv | mod_tcp, mod_http |
| `TcpListener` | socket() + bind() + listen() + accept() | mod_tcp (listen 模式) |

- 构造时自动 `socket()` / `bind()`，析构自动 `close`
- Windows 上封装 `WSAStartup`/`WSACleanup`（内部引用计数）
- send/recv 封装 `sendto`/`recvfrom`/`send`/`recv`，调用方不接触 sockaddr

### net_common.h (header-only)

```cpp
namespace net {
    bool parse_ipv4(const char* str, uint32_t* out);
    std::string format_addr(uint32_t ip, int port);
}
```

## 模块接口规范

每个模块遵循统一签名：

```cpp
// mod_xxx.h
int run_xxx(int argc, char* argv[]);   // 返回 exit code
```

参数约定：
- `argv[0]` 是 flag 字符串（如 `-u`），模块可忽略或用于识别子模式
- `argv[1..]` 是功能参数（如 IP、port 等）
- 模块内部自行校验参数、输出错误提示
- main.cpp 只做 switch-case 分发，不处理模块参数细节

## main.cpp 分发逻辑

```
flag = argv[1]
match flag:
  "-u" / "--udp"     → run_udp(argc-1, argv+1)
  "-t" / "--tcp"     → run_tcp(argc-1, argv+1)
  "-l" / "--listen"  → run_tcp_listen(argc-1, argv+1)
  "-s" / "--scan"    → run_scan(argc-1, argv+1)
  "-d" / "--dns"     → run_dns(argc-1, argv+1)
  ...                → print_usage()
```

## CMakeLists.txt

```cmake
cmake_minimum_required(VERSION 3.10)
project(net-helper LANGUAGES CXX)
set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
file(GLOB_RECURSE SOURCES "src/*.cpp")
add_executable(net-helper ${SOURCES})

# 静态链接，产物独立运行，不依赖外部 DLL
target_link_options(net-helper PRIVATE -static -static-libgcc -static-libstdc++)

if(WIN32)
    target_link_libraries(net-helper ws2_32)
endif()
```

`file(GLOB_RECURSE)` 自动收集 .cpp，新增模块只需添加源文件，无需修改 CMakeLists.txt。

## 模块清单

| Flag | 功能 | 模块文件 | 状态 |
|---|---|---|---|
| `-u` / `--udp` | UDP 收发 | mod_udp.h/cpp | 已实现 |
| `-t` / `--tcp` | TCP 连接 | mod_tcp.h/cpp | 待开发 |
| `-l` / `--listen` | TCP 监听 | mod_tcp.h/cpp | 待开发 |
| `-s` / `--scan` | 端口扫描 | mod_scan.h/cpp | 待开发 |
| `-d` / `--dns` | DNS 查询 | mod_dns.h/cpp | 待开发 |
| `-h` / `--http` | HTTP 请求 | mod_http.h/cpp | 待开发 |
| `-p` / `--ping` | ICMP Ping | mod_ping.h/cpp | 待开发 |

## 设计原则

- **YAGNI**：不提前抽象，只在新增功能时抽取公共代码
- **模块内聚**：每个模块一个 .h/.cpp 对，独立可理解，新增功能不改已有模块
- **跨平台**：socket.h 集中处理平台差异，模块代码不出现 `#ifdef _WIN32`
- **无测试**：手动测试，不引入测试框架
