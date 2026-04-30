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
