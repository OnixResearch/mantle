Task-ID: V4
Covers: bootstrap.gcc40.transition

Status: blocked.

## Blocker

Depends on V2 (gcc-4.0.4 build output). No crunch binary available.

## Required when unblocked

C smoke test:
- Compile `int main(void) { return 0; }` with gcc-4.0.4 output
- Verify exit status 0, output binary exists

C++ smoke test (if C++ language was built):
- Compile trivial C++ program with g++-4.0.4 output
- Verify exit status 0, output binary exists

Record: command, exit status, output path, language tested, fallback status.

Verified: 2026-04-27 (blocker recorded)
