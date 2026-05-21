## Why

The v13 GCC 4.0 native `cc1` source-frontier receipt narrowed the `c-parse.c` include failure to `config.h` plus the first three direct includes from `system.h`: `stdarg.h` and `stddef.h` compile, while adding `stdio.h` reproduces the same two-line truncated include-flood. That is actionable but still too coarse to distinguish `stdio.h` itself from a nested libc include or a `config.h` macro interaction.

## What Changes

- Add a bounded v14 diagnostic slice that probes the `config.h` + `stdio.h` frontier without expanding into broad make-log scraping.
- Update the c-parse source-frontier receipt to schema v14 with compact observed markers.
- Extend fail-closed parity validation and tests so stale v13 evidence or missing v14 markers cannot silently pass.
- Preserve `gcc.4.0` as evidence-backed `partial`; no native compiler/source-build/full GCC correctness is claimed.

## Impact

- Files: `bootstrap/diag-gcc40-c-parse-boundary.ncl`, `bootstrap/evidence/gcc-4.0-native-cc1-build-frontier.json`, `src/bootstrap_parity.rs`, `openspec/specs/bootstrap/spec.md`.
- APIs: none.
- Verification: JSON parse, focused parity tests, CLI parity tests, blocker/source-pin self-tests, parity report confirming `gcc.4.0 partial`, OpenSpec strict validation, and `git diff --check`.
