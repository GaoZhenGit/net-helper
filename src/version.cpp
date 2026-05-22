// version.cpp — Include the build-time generated version header
#include "version.h"
#include "version_gen.h"
#include <cstdio>

void printVersion() {
    printf("%s\n", NETHELPER_VERSION);
}
