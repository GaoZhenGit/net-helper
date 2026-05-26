// socket.h — Cross-platform Socket RAII, this version: base + UdpSocket
#pragma once

#include <cstdint>
#include <cstring>
#include "net_common.h"

#ifdef _WIN32
#include <winsock2.h>
#include <ws2tcpip.h>

using socket_t = SOCKET;
constexpr socket_t INVALID_SOCK = INVALID_SOCKET;
constexpr int SOCK_ERR = SOCKET_ERROR;

inline int sock_errno() { return WSAGetLastError(); }
inline bool would_block() { return WSAGetLastError() == WSAETIMEDOUT; }

inline void sock_close(socket_t fd) { closesocket(fd); }

#else
#include <sys/socket.h>
#include <sys/time.h>
#include <netinet/in.h>
#include <arpa/inet.h>
#include <unistd.h>
#include <errno.h>

using socket_t = int;
constexpr socket_t INVALID_SOCK = -1;
constexpr int SOCK_ERR = -1;

inline int sock_errno() { return errno; }
inline bool would_block() { return errno == EAGAIN || errno == EWOULDBLOCK; }

inline void sock_close(socket_t fd) { close(fd); }
#endif

// RAII base: manages fd lifecycle
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
            net::wsa_cleanup();
        }
    }

protected:
    socket_t fd_;

    bool create(int family, int type, int proto) {
        if (!net::wsa_init()) return false;
        fd_ = socket(family, type, proto);
        return fd_ != INVALID_SOCK;
    }
};

// UDP Socket
class UdpSocket : public Socket {
public:
    // Create UDP socket, bind port=0 (OS assigned), set recv timeout
    bool init(int timeout_ms = 500) {
        if (!create(AF_INET, SOCK_DGRAM, 0))
            return false;

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

    // Send data to ip:port
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

    // Receive data, returns byte count; <0 on error (timeout -1 non-fatal), 0 no data
    // from_ip buffer needs at least INET_ADDRSTRLEN bytes
    int recv_from(void* buf, int buf_len, char* from_ip, int* from_port) {
        struct sockaddr_in from;
        socklen_t fromlen = sizeof(from);

        int n = recvfrom(fd_, static_cast<char*>(buf), buf_len, 0,
                         reinterpret_cast<struct sockaddr*>(&from), &fromlen);
        if (n <= 0) {
            if (n < 0 && would_block()) return -1;  // timeout, non-fatal
            return n;
        }

        if (from_ip)
            inet_ntop(AF_INET, &from.sin_addr, from_ip, INET_ADDRSTRLEN);
        if (from_port)
            *from_port = static_cast<int>(ntohs(from.sin_port));
        return n;
    }
};
