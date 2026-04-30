# Design: tcc musl runtime validation

## Context

The parent derivation now requires `libtcc1.a` and includes an installed-compiler smoke. Runtime proof still depends on prerequisite bootstrap runtime validation for make, musl pass1, and tcc-musl-prep.

## Decisions

### 1. Defer runtime proof until prerequisites are proven

**Choice:** track build/smoke/leakage validation in this follow-up while the parent archives source-level hardening.

**Rationale:** musl-linked TinyCC consumes earlier runtime outputs and must not be promoted independently of them.

### 2. Require compiler-runtime contract

**Choice:** validation must prove installed `tcc`, `tcc-0.9.27-musl`, `libtcc1.a`, and trivial C compilation.

**Rationale:** downstream stages need a compiler that can link generated programs, not just an executable file.

## Validation

Build `bootstrap/tcc-musl.ncl`, smoke compile a trivial C program, verify `libtcc1.a`, scan for leakage, and run OpenSpec validation/gates.
