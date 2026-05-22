// mod_udp.cpp — UDP communication module
#include "mod_udp.h"
#include "socket.h"
#include "net_common.h"
#include <cstdio>
#include <iostream>
#include <string>
#include <thread>
#include <atomic>
#include <mutex>

static std::atomic<bool> s_running{true};

static void udp_receiver(UdpSocket* sock) {
    char buf[65536];

    while (s_running) {
        char from_ip[INET_ADDRSTRLEN];
        int from_port = 0;
        int n = sock->recv_from(buf, sizeof(buf) - 1, from_ip, &from_port);

        if (n < 0) {
            if (!s_running) break;
            continue;
        }
        if (n == 0) continue;

        {
            std::lock_guard<std::mutex> lock(net::out_lock());
            fprintf(stdout, "\n[recv %s:%d %dB] ", from_ip, from_port, n);
            fflush(stdout);
            net::write_stdout(buf, n);
            fprintf(stdout, "\n> ");
            fflush(stdout);
        }
    }
}

int run_udp(int argc, char* argv[]) {
    if (argc < 3) {
        fprintf(stderr, "Usage: net-helper -u <ip|domain> <port>\n");
        return 1;
    }

    int target_port = std::stoi(argv[2]);

    std::string target_ip = net::resolve_first_ipv4(argv[1]);
    if (target_ip.empty())
        target_ip = argv[1];

    UdpSocket sock;
    if (!sock.init(500)) {
        fprintf(stderr, "Failed to create UDP socket\n");
        return 1;
    }

    std::thread recv_thread(udp_receiver, &sock);

    fprintf(stdout, "UDP connected to %s (%s):%d\n", argv[1], target_ip.c_str(), target_port);
    fflush(stdout);

    std::string line;
    while (s_running) {
        {
            std::lock_guard<std::mutex> lock(net::out_lock());
            fprintf(stdout, "> ");
            fflush(stdout);
        }
        if (!std::getline(std::cin, line)) break;
        if (line == "/quit") break;
        if (line.empty()) continue;

        if (!sock.send_to(target_ip.c_str(), target_port, line.c_str(),
                          static_cast<int>(line.size()))) {
            fprintf(stderr, "Send failed\n");
        }
    }

    s_running = false;
    sock.close();
    recv_thread.join();

    return 0;
}
