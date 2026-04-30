# Design: gcc-4.7 runtime validation

## Context

gcc-4.7.4 is downstream of binutils-tcc and gcc-4.0.4. Both runtime evidence chains are now tracked by scoped follow-up changes.

## Decisions

### 1. Validate only after prerequisite evidence

**Choice:** consume completed binutils-tcc and gcc-4.0 runtime validation before claiming gcc-4.7 runtime success.

**Rationale:** gcc-4.7 validation otherwise inherits unproven bootstrap inputs.

## Validation

Build transcript, host-leakage audit, C/C++/C++11 smoke tests, and OpenSpec validation.
