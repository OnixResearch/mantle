# I3 source hardening evidence

Task-ID: I3
Covers: bootstrap.part.musl.1.2.5.full

- Added source provenance and first-consumer comments beside `musl_src`.
- Required non-empty `libc.a`, representative headers, and startup objects (`crt1.o`, `crti.o`, `crtn.o`).
- Made dynamic-linker symlink creation fail closed when `libc.so` exists.
- Added a static link/run smoke using declared GCC and produced musl artifacts.
