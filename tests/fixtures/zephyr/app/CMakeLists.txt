cmake_minimum_required(VERSION 3.20.0)
find_package(Zephyr REQUIRED CONFIG)
project(mantle_minimal_firmware C)
target_sources(app PRIVATE src/main.c)
