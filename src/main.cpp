// main.cpp — net-helper entry point
#include "mod_udp.h"
#include "mod_dns.h"
#include "version.h"
#include <iostream>
#include <cstring>

static void print_usage() {
    std::cout << "net-helper - network diagnostic tool\n\n"
              << "Usage:\n"
              << "  net-helper -u <ip|domain> <port>   UDP send/receive\n"
              << "  net-helper -d <domain>              DNS lookup\n"
              << "  net-helper -v, --version            Show version\n";
}

int main(int argc, char* argv[]) {
    if (argc < 2) {
        print_usage();
        return 1;
    }

    std::string flag = argv[1];

    if (flag == "-u" || flag == "--udp")
        return run_udp(argc - 1, argv + 1);

    if (flag == "-d" || flag == "--dns")
        return run_dns(argc - 1, argv + 1);

    if (flag == "-v" || flag == "--version") {
        printVersion();
        return 0;
    }

    std::cerr << "Unknown flag: " << flag << std::endl;
    print_usage();
    return 1;
}
