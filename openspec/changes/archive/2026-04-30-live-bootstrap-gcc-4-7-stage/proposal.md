# Implement gcc-4.7.4 bootstrap stage

## Why

Parent `live-bootstrap-source-chain` requires `bootstrap/gcc-4.7.ncl` to become
the C++11-capable transition between gcc-4.0.4 and modern GCC. That depends on
the scoped gcc-4.0.4 stage and must not be hidden in the parent task.

## What Changes

- Replace `bootstrap/gcc-4.7.ncl` placeholder with gcc 4.7.4 C and C++ compiler
  outputs built by chain-internal gcc-4.0.4-era inputs.
- Pin gcc 4.7.4 and support artifacts with URL/path, digest, provenance, and
  first-consuming derivation metadata.
- Validate C/C++ compiler smoke tests, C++11 support needed by later GCC, and
  no host/Nix/legacy-provider fallback.

## Scope

- **In scope**: gcc 4.7.4 compiler transition and its directly required support artifacts.
- **Out of scope**: gcc-10.x and final musl/binutils/provider stages.

## Evidence Needed

Completion requires source-pin audit output, build transcript metadata, no-host
leakage audit, fallback markers, and C/C++ smoke tests including a minimal C++11
compile with the gcc-4.7.4 output.
