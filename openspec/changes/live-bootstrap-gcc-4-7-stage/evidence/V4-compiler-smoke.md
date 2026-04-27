Task-ID: V4
Covers: bootstrap.gcc47.transition

Status: blocked.

## Blocker

Depends on V2 (gcc-4.7.4 build output).

## Required when unblocked

- C smoke: compile `int main(void) { return 0; }` with gcc-4.7.4
- C++ smoke: compile trivial C++ program with g++-4.7.4
- C++11 smoke: compile program using range-for, auto, static_assert with `-std=c++11`

Record: command, exit status, output path, language/standard tested.

Verified: 2026-04-27 (blocker recorded)
