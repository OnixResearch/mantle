# I3 source hardening evidence

Task-ID: I3
Covers: bootstrap.part.gmp.6.2.1

- Added source provenance and first-consumer comments beside the `gmp_src` fixed-output fetch.
- Extended the output contract from `libgmp.a` only to include installed `gmp.h`.
- Added a fail-closed compile and run smoke using the produced static GMP library.
