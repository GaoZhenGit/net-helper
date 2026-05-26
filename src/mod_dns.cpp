// mod_dns.cpp — DNS query module implementation
#include "mod_dns.h"
#include "net_common.h"
#include <iostream>

int run_dns(int argc, char* argv[]) {
    if (argc < 2) {
        std::cerr << "Usage: net-helper -d <domain>" << std::endl;
        return 1;
    }

    const char* domain = argv[1];
    auto ips = net::resolve_hostname(domain);

    if (ips.empty()) {
        std::cerr << "Failed to resolve: " << domain << std::endl;
        return 1;
    }

    std::cout << domain << ":" << std::endl;
    for (const auto& ip : ips)
        std::cout << "  " << ip << std::endl;

    return 0;
}
