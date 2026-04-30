# Design: patch 2.5.9 runtime validation

## Context

The parent derivation now checks object creation and exercises `patch` against a simple diff inside the builder. Runtime proof still depends on prerequisite bootstrap executables being repaired and proven.

## Decisions

### 1. Defer runtime proof until prerequisites are proven

**Choice:** track build/smoke/leakage validation in this follow-up while the parent archives its source-level hardening.

**Rationale:** `patch-tcc.ncl` consumes `make-tcc.ncl` and TinyCC runtime paths that are currently tracked by repair/runtime-validation changes.

### 2. Require functional patch smoke

**Choice:** runtime validation must prove more than `patch --version`; it must apply a simple unified diff and verify the result.

**Rationale:** downstream stages need a working patch tool, not just a version-printing binary.

## Validation

Build `bootstrap/patch-tcc.ncl`, smoke `patch --version` plus diff application, scan for leakage, and run OpenSpec validation/gates.
