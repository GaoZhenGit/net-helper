// net_common.h — common network utilities and cross-platform I/O helpers
#pragma once

#include <string>
#include <vector>
#include <cstdint>
#include <cstring>
#include <mutex>

#ifdef _WIN32
#include <winsock2.h>
#include <ws2tcpip.h>
#include <io.h>
#else
#include <arpa/inet.h>
#include <sys/socket.h>
#include <netdb.h>
#include <unistd.h>
#endif

namespace net {

// --- Raw stdout (OS-level, bypasses all buffering) ---

#ifdef _WIN32
inline int raw_stdout(const void* buf, int len) { return _write(1, buf, len); }
#else
inline int raw_stdout(const void* buf, int len) { return write(1, buf, len); }
#endif

// Write all bytes with partial-write retry (thread-safe via external lock)
inline void write_stdout(const void* buf, int len) {
    const char* p = static_cast<const char*>(buf);
    while (len > 0) {
        int n = raw_stdout(p, len);
        if (n <= 0) break;
        p += n;
        len -= n;
    }
}

// Global mutex for serialising stdout across threads
inline std::mutex& out_lock() {
    static std::mutex m;
    return m;
}

// --- Winsock lifecycle (shared across all modules via C++17 inline) ---
#ifdef _WIN32
inline int  s_wsa_ref = 0;

inline bool wsa_init() {
    if (s_wsa_ref++ == 0) {
        WSADATA wsa;
        return WSAStartup(MAKEWORD(2, 2), &wsa) == 0;
    }
    return true;
}

inline void wsa_cleanup() {
    if (--s_wsa_ref == 0)
        WSACleanup();
}
#else
inline bool wsa_init()  { return true; }
inline void wsa_cleanup() {}
#endif

// --- Address helpers ---

inline bool parse_ipv4(const char* str, uint32_t* out) {
    struct in_addr addr;
    if (inet_pton(AF_INET, str, &addr) == 1) {
        *out = addr.s_addr;
        return true;
    }
    return false;
}

inline std::string format_addr(uint32_t ip_net, int port) {
    struct in_addr addr;
    addr.s_addr = ip_net;
    char buf[INET_ADDRSTRLEN];
    inet_ntop(AF_INET, &addr, buf, sizeof(buf));
    return std::string(buf) + ":" + std::to_string(port);
}

// --- DNS resolution ---

inline std::vector<std::string> resolve_hostname(const char* hostname) {
    std::vector<std::string> result;

    if (!wsa_init())
        return result;

    struct addrinfo hints, *res = nullptr;
    std::memset(&hints, 0, sizeof(hints));
    hints.ai_family = AF_UNSPEC;
    hints.ai_socktype = SOCK_STREAM;

    if (getaddrinfo(hostname, nullptr, &hints, &res) != 0 || !res) {
        wsa_cleanup();
        return result;
    }

    for (struct addrinfo* p = res; p; p = p->ai_next) {
        char ip_str[INET6_ADDRSTRLEN];
        void* addr;
        if (p->ai_family == AF_INET)
            addr = &reinterpret_cast<struct sockaddr_in*>(p->ai_addr)->sin_addr;
        else if (p->ai_family == AF_INET6)
            addr = &reinterpret_cast<struct sockaddr_in6*>(p->ai_addr)->sin6_addr;
        else
            continue;
        inet_ntop(p->ai_family, addr, ip_str, sizeof(ip_str));
        result.push_back(ip_str);
    }

    freeaddrinfo(res);
    wsa_cleanup();
    return result;
}

inline std::string resolve_first_ipv4(const char* hostname) {
    auto list = resolve_hostname(hostname);
    for (const auto& ip : list) {
        if (ip.find(':') == std::string::npos)  // IPv4
            return ip;
    }
    return list.empty() ? "" : list[0];
}

} // namespace net
