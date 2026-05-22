// mod_udp.cpp — UDP communication module implementation
#include "mod_udp.h"
#include "socket.h"
#include "net_common.h"
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
    // argv[0] = "-u", argv[1] = ip/domain, argv[2] = port
    if (argc < 3) {
        std::cerr << "Usage: net-helper -u <ip|domain> <port>" << std::endl;
        return 1;
    }

    int target_port = std::stoi(argv[2]);

    // Resolve domain to IP if not already an IP address
    std::string target_ip = net::resolve_first_ipv4(argv[1]);
    if (target_ip.empty())
        target_ip = argv[1];  // fallback: use as-is (might be raw IP)

    UdpSocket sock;
    if (!sock.init(500)) {
        std::cerr << "Failed to create UDP socket" << std::endl;
        return 1;
    }

    std::thread recv_thread(udp_receiver, &sock);

    std::cout << "UDP connected to " << argv[1] << " (" << target_ip << "):"
              << target_port << std::endl;

    std::string line;
    while (s_running) {
        std::cout << "> " << std::flush;
        if (!std::getline(std::cin, line)) break;
        if (line == "/quit") break;
        if (line.empty()) continue;

        if (!sock.send_to(target_ip.c_str(), target_port, line.c_str(),
                          static_cast<int>(line.size()))) {
            std::cerr << "Send failed" << std::endl;
        }
    }

    s_running = false;
    sock.close();          // close socket to unblock recvfrom in receiver thread
    recv_thread.join();

    return 0;
}
