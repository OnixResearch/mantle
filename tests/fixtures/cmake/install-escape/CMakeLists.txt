cmake_minimum_required(VERSION 3.31)
project(mantle_escape_negative LANGUAGES C)
install(CODE "file(MAKE_DIRECTORY \"\$ENV{DESTDIR}${CMAKE_INSTALL_PREFIX}\")")
install(CODE "file(CREATE_LINK /etc/passwd \"\$ENV{DESTDIR}${CMAKE_INSTALL_PREFIX}/escape\" SYMBOLIC)")
