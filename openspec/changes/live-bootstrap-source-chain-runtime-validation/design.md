# Design: source-chain runtime validation

## Context

The source-chain umbrella depends on long-running transition builds. Binutils-tcc runtime validation exceeded the local drain budget in the Mes prerequisite, so downstream gcc and final provider validations must be tracked separately.

## Decisions

### 1. Keep runtime proof as an umbrella follow-up

**Choice:** defer parent V2-V4 to this change while preserving child runtime validation changes for binutils-tcc, gcc-4.0, and gcc-4.7.

**Rationale:** the implementation package can archive while proof work remains explicitly visible and scoped.

## Validation

Complete compiler transitions, final provider stages, and self-build proof/status promotion checks.
