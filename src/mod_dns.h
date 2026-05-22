// mod_dns.h — DNS query module
#pragma once

// DNS query entry point
// argc/argv from main, argv[0] is the flag string ("-d")
// Expected: net-helper -d <domain>
// Returns exit code
int run_dns(int argc, char* argv[]);
