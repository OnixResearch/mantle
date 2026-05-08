# Design: gcc-4.0 runtime validation

## Context

The parent gcc-4.0.4 implementation relies on a functional binutils-tcc output. The binutils-tcc validation was deferred because its first epoch spent longer than the drain budget in the Mes prerequisite.

## Decisions

### 1. Gate gcc-4.0 validation on binutils-tcc runtime evidence

**Choice:** complete or consume `live-bootstrap-binutils-tcc-chain-runtime-validation` before claiming gcc-4.0 runtime validation.

**Rationale:** gcc-4.0 evidence is meaningless if the upstream binutils-tcc chain is not proven.

### 2. Preserve transcript and smoke evidence

**Choice:** use the same transcript fields as the parent and run separate C and C++ smoke tests.

**Rationale:** gcc-4.0 is a compiler transition point, so both language frontends and host-leakage rejection are required.

## Validation

Run build, host-leakage audit, C/C++ compiler smoke, and OpenSpec validation.

## Closeout decision

The change closes as a deterministic negative runtime-validation drain rather than a successful GCC output promotion. V2/V4 evidence records that `gcc-4.0.4` output smoke tests cannot run until the predecessor TinyCC/Mes `libtcc.c` `rc=139` boundary is repaired. The canonical requirement remains conditional: C/C++ smoke evidence is required when `gcc-4.0.4` builds successfully, and this archive preserves the failure-class boundary without claiming that success.
