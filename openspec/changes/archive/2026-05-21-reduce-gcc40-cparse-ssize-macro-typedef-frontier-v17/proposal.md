## Why

Source-frontier v16 showed that defining `ssize_t` before `<stdio.h>` reproduces the GCC 4.0 native `c-parse.o` include-flood failure, while adding the same macro after `<stdio.h>` succeeds. The next smallest useful step is to separate a pre-stdio macro-name collision from a type-definition/order mismatch.

## What Changes

- Add compact diagnostic probes around the v16 seam that compare pre-stdio `#define ssize_t int` with a pre-stdio `typedef int ssize_t` and a harmless/self macro form where practical.
- Update the native `cc1` frontier evidence, parity checks, and bootstrap spec to require the new bounded markers.
- Preserve `gcc.4.0` as evidence-backed `partial`; this does not prove native GCC 4.0 correctness.

## Impact

- Files: `bootstrap/diag-gcc40-c-parse-boundary.ncl`, `bootstrap/evidence/gcc-4.0-native-cc1-build-frontier.json`, `src/bootstrap_parity.rs`, `openspec/specs/bootstrap/spec.md`.
- Testing: diagnostic script generation, focused bootstrap parity tests, CLI parity report, source-pin and blocker-inventory self-tests, OpenSpec strict validation, and whitespace checks.
