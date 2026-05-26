# version.cmake — Generate version_gen.h at build time
# Called by CMakeLists.txt as a pre-build custom target.
#
# Can be overridden: cmake ... -DNETHELPER_VERSION=v2026.12.31.1200
# Otherwise auto-generates: vYYYY.MM.DD.HHMM

if(DEFINED VERSION_OVERRIDE AND NOT VERSION_OVERRIDE STREQUAL "")
    set(VERSION "${VERSION_OVERRIDE}")
    message(STATUS "Version: ${VERSION} (override)")
else()
    string(TIMESTAMP BUILD_TIME "%Y.%m.%d.%H%M")
    set(VERSION "v${BUILD_TIME}")
    message(STATUS "Version: ${VERSION}")
endif()

file(WRITE "${OUTPUT_FILE}" "#pragma once\n#define NETHELPER_VERSION \"${VERSION}\"\n")
