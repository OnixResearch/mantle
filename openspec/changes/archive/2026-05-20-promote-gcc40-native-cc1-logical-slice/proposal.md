## Why

`gcc.4.0` remains a live-bootstrap and Guix blocker. The prior native `cc1` slice proves one bounded arithmetic/control-flow input without TinyCC delegation; the next high-ROI increment is a second bounded frontend semantic shape that exercises logical operators and branch selection while preserving the no-delegation proof and partial-only claim.

## What Changes

- Promote one selected installed-`cc1` logical/control-flow smoke: a small C function using `&&`, `||`, comparison, and a branch/return.
- Update the checked native `cc1` receipt to v2 with the selected logical slice and preserved v1 arithmetic regression.
- Keep TinyCC delegation forbidden for the selected proof input.
- Keep `gcc.4.0` evidence-backed `partial`; do not claim full native GCC 4.0 compiler correctness or parity completion.

## Impact

- Files: `bootstrap/gcc-4.0.ncl`, `bootstrap/evidence/gcc-4.0-native-cc1-arithmetic.json`, `bootstrap/evidence/gcc-4.0-native-boundary.json`, `src/bootstrap_parity.rs`, `openspec/specs/bootstrap/spec.md`.
- Testing: focused GCC 4.0 parity tests, parity snapshot rail, OpenSpec strict validation, and diff whitespace checks.
