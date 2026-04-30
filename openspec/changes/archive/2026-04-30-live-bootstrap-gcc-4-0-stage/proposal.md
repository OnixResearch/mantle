# Implement gcc-4.0.4 bootstrap stage

## Why

Parent `live-bootstrap-source-chain` requires `bootstrap/gcc-4.0.ncl` to become
the mandatory gcc-4.0.4 transition. That stage is blocked on functional
`bootstrap/binutils-tcc.ncl`, so it needs a scoped change that can start after
`live-bootstrap-binutils-tcc-chain` lands.

## What Changes

- Replace `bootstrap/gcc-4.0.ncl` placeholder with gcc 4.0.4 C and C++ compiler
  outputs built by the TinyCC/musl/binutils 2.30 chain.
- Pin gcc 4.0.4 C and C++ source inputs and any carried/generated support artifacts with URL/path,
  digest, provenance, and first-consuming derivation metadata.
- Validate gcc-4.0.4 emits working C and C++ compilers without host leakage or
  legacy-provider fallback.

## Scope

- **In scope**: gcc 4.0.4 C and C++ compiler transition from TinyCC-era binutils to first GCC.
- **Out of scope**: gcc-4.7.4 and later stages.

## Evidence Needed

Completion requires source-pin audit output, build transcript metadata, no-host
leakage audit, host shell/Nix detection over sandbox command transcripts and
proof markers, and C/C++ compiler smoke tests using the gcc-4.0.4 output.
