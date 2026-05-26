cmake_minimum_required(VERSION 3.10)
project(net-helper LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# Let sources include the generated version header from build dir
include_directories(${CMAKE_BINARY_DIR})

file(GLOB_RECURSE SOURCES "src/*.cpp")
add_executable(net-helper ${SOURCES})

# --- Version generation (runs on every build) ---
set(NETHELPER_VERSION "" CACHE STRING "Version override (empty = auto from build time)")
set(VERSION_GEN_H "${CMAKE_BINARY_DIR}/version_gen.h")
add_custom_target(gen_version ALL
    ${CMAKE_COMMAND} -DOUTPUT_FILE=${VERSION_GEN_H}
                     -DVERSION_OVERRIDE=${NETHELPER_VERSION}
                     -P ${CMAKE_SOURCE_DIR}/version.cmake
    COMMENT "Generating version header..."
)
add_dependencies(net-helper gen_version)

# --- Static linking: self-contained binary ---
if(MINGW)
    target_link_options(net-helper PRIVATE -static -static-libgcc -static-libstdc++)
elseif(MSVC)
    target_link_options(net-helper PRIVATE /MT)
else()
    target_link_options(net-helper PRIVATE -static)
endif()

if(WIN32)
    target_link_libraries(net-helper ws2_32)
endif()
