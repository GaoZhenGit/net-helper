// mod_udp.h — UDP communication module
#pragma once

// UDP communication entry point
// argc/argv from main, argv[0] is the flag string ("-u")
// Expected: net-helper -u <ip> <port>
// Returns exit code
int run_udp(int argc, char* argv[]);
