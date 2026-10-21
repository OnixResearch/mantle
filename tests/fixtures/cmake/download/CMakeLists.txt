cmake_minimum_required(VERSION 3.31)
project(mantle_download_negative LANGUAGES C)
file(DOWNLOAD "https://example.invalid/undeclared.tar" "${CMAKE_BINARY_DIR}/download" STATUS ignored)
