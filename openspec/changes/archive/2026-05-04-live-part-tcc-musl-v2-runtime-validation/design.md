# Design: tcc musl v2 runtime validation

## Context

The parent derivation now requires `libtcc1.a` and includes an installed-compiler smoke for the final self-hosted musl TinyCC. Runtime proof still depends on prerequisite bootstrap runtime validation for make, tcc-musl, and rebuilt musl.

## Decisions

### 1. Defer runtime proof until prerequisites are proven

**Choice:** track build/smoke/leakage validation in this follow-up while the parent archives source-level hardening.

**Rationale:** tcc-musl-v2 is downstream of several runtime-blocked stages and must not be promoted independently.

### 2. Require final compiler-runtime contract

**Choice:** validation must prove installed `tcc`, `tcc-0.9.27-musl-v2`, `libtcc1.a`, and trivial C compilation.

**Rationale:** downstream post-musl tools rely on this compiler as the definitive TinyCC stage.

## Validation

Build `bootstrap/tcc-musl-v2.ncl`, smoke compile a trivial C program, verify `libtcc1.a`, scan for leakage, and run OpenSpec validation/gates.
