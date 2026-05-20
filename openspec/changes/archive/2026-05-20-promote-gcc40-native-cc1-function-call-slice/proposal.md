## Why

`gcc.4.0` remains a live-bootstrap and Guix parity blocker. The installed `cc1` has bounded no-TinyCC-delegation evidence for arithmetic/control-flow, logical/control-flow, and local variable assignment/update, but it still lacks a basic function-definition/function-call proof shape.

A tiny helper-call slice is the next low-blast-radius semantic increment: it exercises a distinct frontend/codegen boundary while preserving the partial-only claim and keeping full GCC 4.0 native compiler correctness blocked.

## What Changes

- Promote one selected installed-`cc1` helper-call smoke, such as a tiny C helper function plus a caller that invokes it and returns the result.
- Update the checked native `cc1` receipt to a new schema/selected slice while preserving the arithmetic, logical, and local-variable regressions.
- Keep TinyCC delegation forbidden for the selected proof input.
- Keep `gcc.4.0` evidence-backed `partial`; do not claim full native GCC 4.0 compiler correctness or parity completion.

## Impact

- Files likely touched during implementation: `bootstrap/gcc-4.0.ncl`, `bootstrap/evidence/gcc-4.0-native-cc1-arithmetic.json`, `bootstrap/evidence/gcc-4.0-native-boundary.json`, `bootstrap/evidence/gcc-4.0-placeholder-inventory.json`, `src/bootstrap_parity.rs`, `openspec/specs/bootstrap/spec.md`.
- Testing: focused GCC 4.0 parity tests, CLI parity report, blocker inventory/source-pin checks if line-sensitive metadata shifts, OpenSpec strict validation, and `git diff --check`.
