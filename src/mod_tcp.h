// mod_tcp.h — TCP client module
#pragma once

// TCP connect entry point
// argc/argv from main, argv[0] is the flag string ("-t")
// Expected: net-helper -t <ip|domain> <port>
// Returns exit code
int run_tcp(int argc, char* argv[]);
