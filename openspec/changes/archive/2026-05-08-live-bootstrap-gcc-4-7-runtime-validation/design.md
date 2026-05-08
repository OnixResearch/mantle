# Design: gcc-4.7 runtime validation

## Context

gcc-4.7.4 is downstream of binutils-tcc and gcc-4.0.4. Both runtime evidence chains are now tracked by scoped follow-up changes.

## Decisions

### 1. Validate only after prerequisite evidence

**Choice:** consume completed binutils-tcc and gcc-4.0 runtime validation before claiming gcc-4.7 runtime success.

**Rationale:** gcc-4.7 validation otherwise inherits unproven bootstrap inputs.

## Validation

Build transcript, host-leakage audit, C/C++/C++11 smoke tests, and OpenSpec validation.

### 2. Close as blocker-preservation evidence

**Choice:** archive this validation as an explicit negative/blocked status rather than leaving an umbrella task open.

**Rationale:** the direct prerequisite `gcc-4.0.4` provider has no runtime-validated output. Keeping gcc-4.7 conditional documents the blocker chain and prevents accidental promotion through host GCC, Nix, or another legacy provider.
