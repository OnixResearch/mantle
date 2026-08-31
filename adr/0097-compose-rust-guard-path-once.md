# ADR 0097: Compose the Rust guard path once

## Status

Accepted (2026-08-31)

## Context

ADR 0096 added bounded unavailable-tool adapters to ordinary cargo-free toolchain
preparation. V94 failed before protected stage1 because fixed-point stages used
a second setup sequence. That sequence wrote toolchain and host-tool aliases,
then constructed action authority without writing the adapters.

The authority builder correctly rejected the missing `emcc` adapter. The
failure exposed two imperative-shell composition roots for one security
boundary.

## Decision Drivers

- Keep alias bytes and action authority in the same preparation transaction.
- Apply one deterministic ordering to ordinary and fixed-point execution.
- Prevent future alias classes from reaching only one execution route.
- Construct authority only after all guarded executables exist.
- Keep pure alias selection and validation separate from file effects.

## Decision

Use `prepare_receipt_bound_guard_path` as the single shell composition function
for both ordinary cargo-free builds and fixed-point stages.

The function performs these steps in order:

1. Write validated toolchain aliases and construct the one-entry `PATH`.
2. Write source-built host-tool aliases.
3. If an explicit closure exists, write missing optional-tool policy adapters.
4. Return the guarded `PATH` only after every executable is materialized.

Construct Rust child-action authority after this function returns.

Tests that check fixed executable authority must call the shared composition
function instead of recreating its steps.

## Alternatives Considered

### Add one adapter call to fixed-point execution

Rejected. Two independent sequences would drift again when another alias class
is added.

### Let authority creation write missing files

Rejected. Authority construction should measure and validate prepared effects,
not perform hidden publication.

### Treat missing adapters as disabled authority

Rejected. The action can still issue the probe. The executable must exist and
be bound before supervision starts.

## Consequences

- Ordinary and fixed-point builds receive the same guarded executables.
- Authority measures files only after deterministic materialization.
- A missing or modified adapter still fails closed.
- The imperative shell has one visible owner for guarded PATH publication.
- V94 remains failed evidence. A fresh promoted proof must validate ordering.
