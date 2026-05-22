// mod_tcp.cpp — TCP client module implementation
#include "mod_tcp.h"
#include "socket.h"
#include "net_common.h"
#include <cstdio>
#include <iostream>
#include <string>
#include <thread>
#include <atomic>
#include <mutex>

static std::atomic<bool> s_running{true};
static std::mutex s_print_mutex;

// Write all bytes to stdout, retrying partial writes
static void raw_write_all(const char* p, int len) {
    while (len > 0) {
        int n = net::raw_stdout(p, len);
        if (n <= 0) break;
        p += n;
        len -= n;
    }
}

static void tcp_receiver(TcpSocket* sock) {
    char buf[65536];

    while (s_running) {
        int n = sock->recv(buf, sizeof(buf) - 1);

        if (n == -2) {                 // timeout
            if (!s_running) break;
            continue;
        }

        if (n < 0) {                   // connection lost (RST, etc.)
            std::lock_guard<std::mutex> lock(s_print_mutex);
            fprintf(stdout, "\nConnection lost\n");
            fflush(stdout);
            s_running = false;
            break;
        }

        if (n == 0) {                  // graceful close (FIN)
            std::lock_guard<std::mutex> lock(s_print_mutex);
            fprintf(stdout, "\nConnection closed by remote\n");
            fflush(stdout);
            s_running = false;
            break;
        }

        {
            std::lock_guard<std::mutex> lock(s_print_mutex);
            fprintf(stdout, "\n[recv %d bytes] ", n);
            fflush(stdout);
            raw_write_all(buf, n);
            fprintf(stdout, "\n> ");
            fflush(stdout);
        }
    }
}

int run_tcp(int argc, char* argv[]) {
    // Force unbuffered stdout for reliable console output
    setvbuf(stdout, nullptr, _IONBF, 0);

    if (argc < 3) {
        fprintf(stderr, "Usage: net-helper -t <ip|domain> <port>\n");
        return 1;
    }

    int target_port = std::stoi(argv[2]);

    std::string target_ip = net::resolve_first_ipv4(argv[1]);
    if (target_ip.empty())
        target_ip = argv[1];

    TcpSocket sock;
    if (!sock.connect(target_ip.c_str(), target_port)) {
        fprintf(stderr, "Failed to connect to %s:%d\n", argv[1], target_port);
        return 1;
    }

    std::thread recv_thread(tcp_receiver, &sock);

    fprintf(stdout, "Connected to %s (%s):%d\n", argv[1], target_ip.c_str(), target_port);
    fflush(stdout);

    std::string line;
    while (s_running) {
        {
            std::lock_guard<std::mutex> lock(s_print_mutex);
            fprintf(stdout, "> ");
            fflush(stdout);
        }
        if (!std::getline(std::cin, line)) break;
        if (line == "/quit") break;

        line += "\n";
        if (!sock.send(line.c_str(), static_cast<int>(line.size()))) {
            fprintf(stderr, "Send failed\n");
            s_running = false;
            break;
        }
    }

    s_running = false;
    sock.close();
    recv_thread.join();

    return 0;
}
