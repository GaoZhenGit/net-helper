// net_common.h — common network utility functions
#pragma once

#include <string>
#include <cstdint>

#ifdef _WIN32
#include <winsock2.h>
#include <ws2tcpip.h>
#else
#include <arpa/inet.h>
#include <sys/socket.h>
#endif

namespace net {

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

} // namespace net
