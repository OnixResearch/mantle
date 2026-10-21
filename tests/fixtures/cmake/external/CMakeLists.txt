cmake_minimum_required(VERSION 3.31)
project(mantle_external_negative LANGUAGES C)
include(ExternalProject)
ExternalProject_Add(undeclared URL "https://example.invalid/undeclared.tar")
