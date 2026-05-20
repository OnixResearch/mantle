## Why

`gcc.4.0` remains a live-bootstrap and Guix parity blocker. The installed `cc1` now has bounded no-TinyCC-delegation proof paths for arithmetic/control-flow and logical/control-flow inputs, but that still leaves basic local-variable declaration/assignment/update behavior outside the native evidence frontier.

A small local-variable slice is the next low-blast-radius increment: it exercises a distinct frontend/codegen shape while preserving the partial-only claim and keeping full GCC 4.0 compiler correctness blocked.

## What Changes

- Promote one selected installed-`cc1` local-variable assignment/update smoke, such as a tiny C function with local `int` declarations, assignment, reassignment, and return.
- Update the checked native `cc1` receipt to a new schema/selected slice while preserving the v1 arithmetic and v2 logical regressions.
- Keep TinyCC delegation forbidden for the selected proof input.
- Keep `gcc.4.0` evidence-backed `partial`; do not claim full native GCC 4.0 compiler correctness or parity completion.

## Impact

- Files likely touched during implementation: `bootstrap/gcc-4.0.ncl`, `bootstrap/evidence/gcc-4.0-native-cc1-arithmetic.json`, `bootstrap/evidence/gcc-4.0-native-boundary.json`, `bootstrap/evidence/gcc-4.0-placeholder-inventory.json`, `src/bootstrap_parity.rs`, `openspec/specs/bootstrap/spec.md`.
- Testing: focused GCC 4.0 parity tests, CLI parity snapshot, OpenSpec strict validation, and `git diff --check`.
