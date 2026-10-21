cmake_minimum_required(VERSION 3.31)
project(mantle_leak_negative LANGUAGES C)
find_path(HOST_LEAK unistd.h PATHS /usr/include NO_DEFAULT_PATH NO_CMAKE_FIND_ROOT_PATH REQUIRED)
message(FATAL_ERROR "host header leaked: ${HOST_LEAK}")
