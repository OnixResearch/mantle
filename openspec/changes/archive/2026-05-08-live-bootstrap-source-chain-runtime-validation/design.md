# Design: source-chain runtime validation

## Context

The source-chain umbrella depends on long-running transition builds. Binutils-tcc runtime validation exceeded the local drain budget in the Mes prerequisite, so downstream gcc and final provider validations must be tracked separately.

## Decisions

### 1. Keep runtime proof as an umbrella follow-up

**Choice:** defer parent V2-V4 to this change while preserving child runtime validation changes for binutils-tcc, gcc-4.0, and gcc-4.7.

**Rationale:** the implementation package can archive while proof work remains explicitly visible and scoped.

## Validation

Complete compiler transitions, final provider stages, and self-build proof/status promotion checks.

## Closeout decision

This change closes as a blocker-preservation runtime-validation umbrella, not as a full-source bootstrap promotion. The accepted result is that transition blockers are explicit: binutils-tcc has archived bridge-boundary evidence, gcc-4.0 has archived negative TinyCC/Mes `libtcc.c rc=139` evidence, and gcc-4.7 remains the next active runtime-validation follow-up. Final-provider normalization and source-built provider proof remain conditional on those prerequisites; no final provider or full-source status promotion is claimed by this archive.
