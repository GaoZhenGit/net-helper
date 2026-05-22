// net_common.h — common network utility functions
#pragma once

#include <string>
#include <vector>
#include <cstdint>
#include <cstring>

#ifdef _WIN32
#include <winsock2.h>
#include <ws2tcpip.h>
#else
#include <arpa/inet.h>
#include <sys/socket.h>
#include <netdb.h>
#endif

namespace net {

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

// Parse IPv4 string -> network-byte-order uint32_t
// Returns true on success
inline bool parse_ipv4(const char* str, uint32_t* out) {
    struct in_addr addr;
    if (inet_pton(AF_INET, str, &addr) == 1) {
        *out = addr.s_addr;
        return true;
    }
    return false;
}

// Network-byte-order uint32_t + port -> "a.b.c.d:port" string
inline std::string format_addr(uint32_t ip_net, int port) {
    struct in_addr addr;
    addr.s_addr = ip_net;
    char buf[INET_ADDRSTRLEN];
    inet_ntop(AF_INET, &addr, buf, sizeof(buf));
    return std::string(buf) + ":" + std::to_string(port);
}

// Resolve hostname using system DNS, returns all IP strings
// Returns empty vector on failure
inline std::vector<std::string> resolve_hostname(const char* hostname) {
    std::vector<std::string> result;

    if (!wsa_init())
        return result;

    struct addrinfo hints, *res = nullptr;

    std::memset(&hints, 0, sizeof(hints));
    hints.ai_family = AF_UNSPEC;     // both IPv4 and IPv6
    hints.ai_socktype = SOCK_STREAM;

    if (getaddrinfo(hostname, nullptr, &hints, &res) != 0 || !res)
        return result;

    for (struct addrinfo* p = res; p; p = p->ai_next) {
        char ip_str[INET6_ADDRSTRLEN];
        void* addr;
        if (p->ai_family == AF_INET) {
            addr = &reinterpret_cast<struct sockaddr_in*>(p->ai_addr)->sin_addr;
        } else if (p->ai_family == AF_INET6) {
            addr = &reinterpret_cast<struct sockaddr_in6*>(p->ai_addr)->sin6_addr;
        } else {
            continue;
        }
        inet_ntop(p->ai_family, addr, ip_str, sizeof(ip_str));
        result.push_back(ip_str);
    }

    freeaddrinfo(res);
    wsa_cleanup();
    return result;
}

// Resolve hostname, return first IPv4 address or empty string on failure
inline std::string resolve_first_ipv4(const char* hostname) {
    auto list = resolve_hostname(hostname);
    for (const auto& ip : list) {
        // IPv4 addresses don't contain ':'
        if (ip.find(':') == std::string::npos)
            return ip;
    }
    // fallback: return first result (might be IPv6 or empty)
    return list.empty() ? "" : list[0];
}

} // namespace net
