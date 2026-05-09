# I1 upstream ordering evidence

Task-ID: I1
Covers: bootstrap.part.musl.1.2.5.full

`bootstrap/musl-full.ncl` tracks musl 1.2.5 as the final libc after the GCC 10 boundary and before later full-toolchain consumers. The source pin now records live-bootstrap provenance and first-consumer notes beside the fixed-output fetch.
