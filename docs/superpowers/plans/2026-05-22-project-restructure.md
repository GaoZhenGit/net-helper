# 工程结构重构 实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 将现有单文件 main.cpp 重构为分层架构（socket.h → net_common.h → mod_udp.cpp → main.cpp），仅迁移 UDP 功能，不新增其他模块。

**Architecture:** 核心层 header-only（socket.h + net_common.h），功能模块 .h/.cpp 对（mod_udp），入口 main.cpp 做 flag 分发。新增模块只需加文件 + main.cpp 加一行分发。

**Tech Stack:** C++17, MinGW GCC 12.1.0, CMake 3.28, Winsock2 (Windows) / BSD sockets (Linux)

---

## 文件变更清单

| 操作 | 文件 | 职责 |
|---|---|---|
| 创建 | `src/socket.h` | 跨平台 Socket RAII 基类 + UdpSocket |
| 创建 | `src/net_common.h` | 地址解析/格式化工具 |
| 创建 | `src/mod_udp.h` | UDP 模块声明 |
| 创建 | `src/mod_udp.cpp` | UDP 模块实现（从原 main.cpp 迁移） |
| 重写 | `src/main.cpp` | 入口：flag 解析 + 分发 |
| 重写 | `CMakeLists.txt` | GLOB_RECURSE + 静态链接 |
| 删除 | `main.cpp` | 移到 src/ 内 |

---

### Task 1: 创建 src/ 目录并移动原文件

**Files:**
- Create: `src/` (directory)
- Move: `main.cpp` → `src/main.cpp.bak`

- [ ] **Step 1: 创建 src/ 目录，备份原文件**

```powershell
mkdir -p D:\project\net-helper\src
Move-Item D:\project\net-helper\main.cpp D:\project\net-helper\src\main.cpp.bak
```

---

### Task 2: 编写 src/socket.h

**Files:**
- Create: `src/socket.h`

- [ ] **Step 1: 创建 socket.h — 跨平台 Socket RAII 封装**

```cpp
// socket.h — 跨平台 Socket RAII，本次仅实现基类 + UdpSocket
#pragma once

#include <cstring>
#include <stdexcept>

#ifdef _WIN32
#include <winsock2.h>
#include <ws2tcpip.h>
#pragma comment(lib, "ws2_32.lib")

namespace {
    static int s_wsa_ref = 0;
    inline bool wsa_init() {
        if (s_wsa_ref++ == 0)
            return WSAStartup(MAKEWORD(2, 2), nullptr) == 0;
        return true;
    }
    inline void wsa_cleanup() {
        if (--s_wsa_ref == 0)
            WSACleanup();
    }
}

using socket_t = SOCKET;
constexpr socket_t INVALID_SOCK = INVALID_SOCKET;
#define SOCK_ERR SOCKET_ERROR

inline int sock_errno() { return WSAGetLastError(); }
inline bool would_block() { return WSAGetLastError() == WSAETIMEDOUT; }

inline void sock_close(socket_t fd) { closesocket(fd); }

#else
#include <sys/socket.h>
#include <netinet/in.h>
#include <arpa/inet.h>
#include <unistd.h>
#include <fcntl.h>
#include <errno.h>

using socket_t = int;
constexpr socket_t INVALID_SOCK = -1;
#define SOCK_ERR (-1)

inline int sock_errno() { return errno; }
inline bool would_block() { return errno == EAGAIN || errno == EWOULDBLOCK; }

inline void sock_close(socket_t fd) { close(fd); }
#endif

// RAII 基类：管理 fd 生命周期
class Socket {
public:
    Socket() : fd_(INVALID_SOCK) {}
    ~Socket() { close(); }

    Socket(Socket&& other) noexcept : fd_(other.fd_) {
        other.fd_ = INVALID_SOCK;
    }

    Socket(const Socket&) = delete;
    Socket& operator=(const Socket&) = delete;
    Socket& operator=(Socket&& other) noexcept {
        if (this != &other) {
            close();
            fd_ = other.fd_;
            other.fd_ = INVALID_SOCK;
        }
        return *this;
    }

    bool valid() const { return fd_ != INVALID_SOCK; }
    socket_t raw_fd() const { return fd_; }

    void close() {
        if (fd_ != INVALID_SOCK) {
            sock_close(fd_);
            fd_ = INVALID_SOCK;
#ifdef _WIN32
            wsa_cleanup();
#endif
        }
    }

protected:
    socket_t fd_;

    bool create(int family, int type, int proto) {
#ifdef _WIN32
        if (!wsa_init()) return false;
#endif
        fd_ = socket(family, type, proto);
        return fd_ != INVALID_SOCK;
    }
};

// UDP Socket
class UdpSocket : public Socket {
public:
    // 创建 UDP socket，绑定 port=0（系统分配），设置接收超时
    bool init(int timeout_ms = 500) {
        if (!create(AF_INET, SOCK_DGRAM, 0))
            return false;

        // bind port 0 → OS assigns ephemeral port
        struct sockaddr_in local;
        std::memset(&local, 0, sizeof(local));
        local.sin_family = AF_INET;
        local.sin_addr.s_addr = htonl(INADDR_ANY);
        local.sin_port = 0;
        if (bind(fd_, reinterpret_cast<struct sockaddr*>(&local), sizeof(local)) < 0) {
            close();
            return false;
        }

        // set recv timeout
#ifdef _WIN32
        int to = timeout_ms;
        setsockopt(fd_, SOL_SOCKET, SO_RCVTIMEO, reinterpret_cast<const char*>(&to), sizeof(to));
#else
        struct timeval tv;
        tv.tv_sec = timeout_ms / 1000;
        tv.tv_usec = (timeout_ms % 1000) * 1000;
        setsockopt(fd_, SOL_SOCKET, SO_RCVTIMEO, &tv, sizeof(tv));
#endif
        return true;
    }

    // 发送数据到指定 ip:port
    bool send_to(const char* ip, int port, const void* data, int len) {
        struct sockaddr_in target;
        std::memset(&target, 0, sizeof(target));
        target.sin_family = AF_INET;
        target.sin_port = htons(static_cast<uint16_t>(port));
        inet_pton(AF_INET, ip, &target.sin_addr);

        int n = sendto(fd_, static_cast<const char*>(data), len, 0,
                       reinterpret_cast<struct sockaddr*>(&target), sizeof(target));
        return n != SOCK_ERR;
    }

    // 接收数据，返回字节数；<0 表示错误(超时返回-1但非致命)，0 表示无数据
    // from_ip 缓冲区至少 INET_ADDRSTRLEN 字节
    int recv_from(void* buf, int buf_len, char* from_ip, int* from_port) {
        struct sockaddr_in from;
        socklen_t fromlen = sizeof(from);

        int n = recvfrom(fd_, static_cast<char*>(buf), buf_len, 0,
                         reinterpret_cast<struct sockaddr*>(&from), &fromlen);
        if (n <= 0) {
            if (n < 0 && would_block()) return -1;  // timeout, non-fatal
            return n;  // error or 0
        }

        if (from_ip)
            inet_ntop(AF_INET, &from.sin_addr, from_ip, INET_ADDRSTRLEN);
        if (from_port)
            *from_port = static_cast<int>(ntohs(from.sin_port));
        return n;
    }
};
```

---

### Task 3: 编写 src/net_common.h

**Files:**
- Create: `src/net_common.h`

- [ ] **Step 1: 创建 net_common.h — 地址解析工具**

```cpp
// net_common.h — 通用网络工具函数
#pragma once

#include <string>
#include <cstdint>
#include <cstring>

#ifdef _WIN32
#include <winsock2.h>
#include <ws2tcpip.h>
#else
#include <arpa/inet.h>
#include <sys/socket.h>
#endif

namespace net {

// 解析 IPv4 地址字符串 → 网络字节序 uint32_t
// 成功返回 true，失败返回 false
inline bool parse_ipv4(const char* str, uint32_t* out) {
    struct in_addr addr;
    if (inet_pton(AF_INET, str, &addr) == 1) {
        *out = addr.s_addr;
        return true;
    }
    return false;
}

// 网络字节序 uint32_t + 端口 → "a.b.c.d:port" 字符串
inline std::string format_addr(uint32_t ip_net, int port) {
    struct in_addr addr;
    addr.s_addr = ip_net;
    char buf[INET_ADDRSTRLEN];
    inet_ntop(AF_INET, &addr, buf, sizeof(buf));
    return std::string(buf) + ":" + std::to_string(port);
}

} // namespace net
```

---

### Task 4: 编写 src/mod_udp.h

**Files:**
- Create: `src/mod_udp.h`

- [ ] **Step 1: 创建 mod_udp.h — UDP 模块声明**

```cpp
// mod_udp.h — UDP 通信模块
#pragma once

// UDP 通信入口
// argc/argv 从 main 传来，argv[0] 为 flag 字符串 ("-u")
// 期望: net-helper -u <ip> <port>
// 返回 exit code
int run_udp(int argc, char* argv[]);
```

---

### Task 5: 编写 src/mod_udp.cpp

**Files:**
- Create: `src/mod_udp.cpp`

- [ ] **Step 1: 创建 mod_udp.cpp — UDP 模块实现**

```cpp
// mod_udp.cpp — UDP 通信模块实现
#include "mod_udp.h"
#include "socket.h"
#include <iostream>
#include <string>
#include <thread>
#include <atomic>

static std::atomic<bool> s_running{true};

static void udp_receiver(UdpSocket* sock) {
    char buf[65536];

    while (s_running) {
        char from_ip[INET_ADDRSTRLEN];
        int from_port = 0;
        int n = sock->recv_from(buf, sizeof(buf) - 1, from_ip, &from_port);

        if (n < 0) {
            if (!s_running) break;
            continue;  // timeout, retry
        }
        if (n == 0) continue;

        buf[n] = '\0';
        std::cout << "\n[recv " << from_ip << ":" << from_port
                  << "] " << buf << "\n> " << std::flush;
    }
}

int run_udp(int argc, char* argv[]) {
    // argv[0] = "-u", argv[1] = ip, argv[2] = port
    if (argc < 3) {
        std::cerr << "Usage: net-helper -u <ip> <port>" << std::endl;
        return 1;
    }

    const char* target_ip = argv[1];
    int target_port = std::stoi(argv[2]);

    UdpSocket sock;
    if (!sock.init(500)) {
        std::cerr << "Failed to create UDP socket" << std::endl;
        return 1;
    }

    std::thread recv_thread(udp_receiver, &sock);

    std::cout << "UDP connected to " << target_ip << ":" << target_port << std::endl;

    std::string line;
    while (s_running) {
        std::cout << "> " << std::flush;
        if (!std::getline(std::cin, line)) break;
        if (line == "/quit") break;
        if (line.empty()) continue;

        if (!sock.send_to(target_ip, target_port, line.c_str(),
                          static_cast<int>(line.size()))) {
            std::cerr << "Send failed" << std::endl;
        }
    }

    s_running = false;
    sock.close();          // close socket to unblock recvfrom in receiver thread
    recv_thread.join();

    return 0;
}
```

---

### Task 6: 重写 src/main.cpp

**Files:**
- Create: `src/main.cpp`

- [ ] **Step 1: 创建 main.cpp — 入口与 flag 分发**

```cpp
// main.cpp — net-helper 入口
#include "mod_udp.h"
#include <iostream>

static void print_usage() {
    std::cout << "net-helper - network diagnostic tool\n\n"
              << "Usage:\n"
              << "  net-helper -u <ip> <port>     UDP send/receive\n";
    // 后续功能在此追加 usage 行
}

int main(int argc, char* argv[]) {
    if (argc < 2) {
        print_usage();
        return 1;
    }

    std::string flag = argv[1];

    if (flag == "-u" || flag == "--udp")
        return run_udp(argc - 1, argv + 1);

    std::cerr << "Unknown flag: " << flag << std::endl;
    print_usage();
    return 1;
}
```

---

### Task 7: 更新 CMakeLists.txt

**Files:**
- Overwrite: `CMakeLists.txt`

- [ ] **Step 1: 更新 CMakeLists.txt**

```cmake
cmake_minimum_required(VERSION 3.10)
project(net-helper LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

file(GLOB_RECURSE SOURCES "src/*.cpp")
add_executable(net-helper ${SOURCES})

# 静态链接，产物独立运行
target_link_options(net-helper PRIVATE -static -static-libgcc -static-libstdc++)

if(WIN32)
    target_link_libraries(net-helper ws2_32)
endif()
```

---

### Task 8: 编译验证

**Files:**
- (无变更)

- [ ] **Step 1: 清理旧 build 产物并重新构建**

```bash
cd D:\project\net-helper && rm -rf build && mkdir build && cd build && cmake -G "MinGW Makefiles" .. && cmake --build .
```
Expected: 编译成功，输出 `net-helper.exe`

---

### Task 9: 功能验证

**Files:**
- (无变更)

- [ ] **Step 1: 验证参数提示正常**

```bash
cd D:\project\net-helper\build && ./net-helper.exe
```
Expected: 打印 usage 信息，exit code 1

- [ ] **Step 2: 验证 UDP 收发正常**

```bash
cd D:\project\net-helper\build && (echo "usee-test"; sleep 2; echo "/quit") | ./net-helper.exe -u 202.108.144.21 2077
```
Expected: 发送后收到服务器回包 `[recv 202.108.144.21:2077] usee-test`

- [ ] **Step 3: 验证静态链接，产物可独立运行**

```bash
cd D:\project\net-helper\build && objdump -p net-helper.exe | grep "DLL Name" | head -5
```
Expected: 仅依赖系统 DLL（kernel32.dll, msvcrt.dll 等），无 libgcc/libstdc++/libwinpthread DLL

---

### Task 10: 清理备份文件

**Files:**
- Delete: `src/main.cpp.bak`

- [ ] **Step 1: 删除备份文件**

```powershell
Remove-Item D:\project\net-helper\src\main.cpp.bak
```
