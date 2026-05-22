#include <iostream>
#include <string>
#include <thread>
#include <atomic>
#include <cstring>

#ifdef _WIN32
#include <winsock2.h>
#include <ws2tcpip.h>
#pragma comment(lib, "ws2_32.lib")
#define socklen_t int
#define close_socket closesocket
#else
#include <sys/socket.h>
#include <netinet/in.h>
#include <arpa/inet.h>
#include <unistd.h>
#define close_socket close
#endif

std::atomic<bool> running{true};

void receiver(int sock) {
    char buffer[65536];

    while (running) {
        struct sockaddr_in from;
        socklen_t fromlen = sizeof(from);
        int n = recvfrom(sock, buffer, sizeof(buffer) - 1, 0,
                         reinterpret_cast<struct sockaddr*>(&from), &fromlen);
        if (n < 0) {
#ifdef _WIN32
            if (WSAGetLastError() == WSAETIMEDOUT) continue;
#else
            if (errno == EAGAIN || errno == EWOULDBLOCK) continue;
#endif
            if (!running) break;
            continue;
        }
        if (n == 0) continue;

        buffer[n] = '\0';
        char from_ip[INET_ADDRSTRLEN];
        inet_ntop(AF_INET, &from.sin_addr, from_ip, sizeof(from_ip));
        std::cout << "\n[recv " << from_ip << ":" << ntohs(from.sin_port)
                  << "] " << buffer << "\n> " << std::flush;
    }
}

int main(int argc, char* argv[]) {
    if (argc != 4 || std::string(argv[1]) != "-u") {
        std::cerr << "Usage: " << argv[0] << " -u <ip> <port>" << std::endl;
        return 1;
    }

    std::string target_ip = argv[2];
    int target_port = std::stoi(argv[3]);

#ifdef _WIN32
    WSADATA wsaData;
    if (WSAStartup(MAKEWORD(2, 2), &wsaData) != 0) {
        std::cerr << "WSAStartup failed" << std::endl;
        return 1;
    }
#endif

    int sock = socket(AF_INET, SOCK_DGRAM, 0);
    if (sock < 0) {
        std::cerr << "Failed to create socket" << std::endl;
#ifdef _WIN32
        WSACleanup();
#endif
        return 1;
    }

    // Bind to port 0 to get an ephemeral port assigned immediately,
    // so the receiver thread can start listening right away.
    struct sockaddr_in local;
    std::memset(&local, 0, sizeof(local));
    local.sin_family = AF_INET;
    local.sin_addr.s_addr = htonl(INADDR_ANY);
    local.sin_port = 0;
    if (bind(sock, reinterpret_cast<struct sockaddr*>(&local), sizeof(local)) < 0) {
        std::cerr << "Failed to bind socket" << std::endl;
        close_socket(sock);
#ifdef _WIN32
        WSACleanup();
#endif
        return 1;
    }

    // 500ms receive timeout so the receiver thread can check the exit flag
    int timeout = 500;
#ifdef _WIN32
    setsockopt(sock, SOL_SOCKET, SO_RCVTIMEO,
               reinterpret_cast<const char*>(&timeout), sizeof(timeout));
#else
    struct timeval tv = {0, 500000};
    setsockopt(sock, SOL_SOCKET, SO_RCVTIMEO, &tv, sizeof(tv));
#endif

    struct sockaddr_in target;
    std::memset(&target, 0, sizeof(target));
    target.sin_family = AF_INET;
    target.sin_port = htons(target_port);
    inet_pton(AF_INET, target_ip.c_str(), &target.sin_addr);

    std::thread recv_thread(receiver, sock);

    std::cout << "UDP connected to " << target_ip << ":" << target_port << std::endl;

    std::string line;
    while (running) {
        std::cout << "> " << std::flush;
        if (!std::getline(std::cin, line)) break;
        if (line == "/quit") break;
        if (line.empty()) continue;

        int sent = sendto(sock, line.c_str(), static_cast<int>(line.size()), 0,
                          reinterpret_cast<struct sockaddr*>(&target), sizeof(target));
        if (sent < 0) {
            std::cerr << "Send failed" << std::endl;
        }
    }

    running = false;
    close_socket(sock);
    recv_thread.join();

#ifdef _WIN32
    WSACleanup();
#endif

    return 0;
}
