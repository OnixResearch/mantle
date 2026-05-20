# Change: Promote GCC 4.0 native cc1 pointer-deref slice

## Why

GCC 4.0 remains evidence-backed partial. The installed native `cc1` has bounded no-TinyCC-delegation semantic evidence for arithmetic, logical/control-flow, local variables, helper calls, array indexing, and struct fields. A small pointer-address/dereference slice adds inspectable coverage of another core C frontend/codegen shape while preserving the existing non-claim boundary.

## What Changes

- Add one bounded installed-`cc1` input that uses a local `int`, a pointer to that local, pointer dereference store/load, and return.
- Emit a new no-TinyCC-delegation object marker for the pointer slice.
- Advance the native `cc1` semantic receipt schema to v7 while preserving v1-v6 regressions.
- Update parity validation/tests to fail closed on schema, marker, digest, input-fragment, or regression drift.
- Keep `gcc.4.0` partial and explicitly avoid claiming full native compiler/source-build correctness.

## Non-goals

- No proof of general pointer semantics, alias analysis, pointer arithmetic, arrays through pointers, ABI/layout correctness, native source-build progress, or full GCC correctness.
- No change to live-bootstrap/Guix/StageX parity completion status.
