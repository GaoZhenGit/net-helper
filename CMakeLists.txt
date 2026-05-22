cmake_minimum_required(VERSION 3.10)
project(net-helper LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

file(GLOB_RECURSE SOURCES "src/*.cpp")
add_executable(net-helper ${SOURCES})

# Static linking: self-contained binary
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
