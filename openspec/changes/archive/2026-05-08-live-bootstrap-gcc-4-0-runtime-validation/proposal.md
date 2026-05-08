# Complete gcc-4.0 runtime validation

## Why

Parent change `live-bootstrap-gcc-4-0-stage` implemented the gcc-4.0.4 derivation surface, but V2-V4 depend on completing the binutils-tcc runtime validation and transcript checker work. The local drain created `live-bootstrap-binutils-tcc-chain-runtime-validation` for the long-running binutils prerequisite.

## What Changes

- Build `bootstrap/gcc-4.0.ncl` after binutils-tcc runtime validation is available.
- Record transcript fields for command, provider, exit status, output path/failure class, fallback status/event marker, and placeholder rejection.
- Run host-leakage checks and C/C++ compiler smoke tests against the gcc-4.0.4 output.

## Scope

In scope: validation evidence and narrowly scoped fixes required for gcc-4.0.4 validation.

Out of scope: gcc-4.7 and later stages.
