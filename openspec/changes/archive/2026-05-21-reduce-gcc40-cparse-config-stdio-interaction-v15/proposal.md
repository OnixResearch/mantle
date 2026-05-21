## Why

The v14 GCC 4.0 native `cc1` source-frontier receipt proved `<stdio.h>` alone compiles under the c-parse flags, while `config.h` plus `<stdio.h>` reproduces the two-line 2047-byte truncated include-flood. The six-undef `config-undef6.h` variant also compiles, so the next actionable uncertainty is which bounded `config.h` macro/typedef interaction poisons `<stdio.h>`.

## What Changes

- Add a bounded v15 diagnostic slice that bisects the `config.h` + `<stdio.h>` interaction with compact config-fragment probes.
- Update the c-parse source-frontier receipt to schema v15 with observed config/stdio markers.
- Extend fail-closed parity validation and regressions so stale v14 evidence or missing v15 markers cannot pass.
- Preserve `gcc.4.0` as evidence-backed `partial`; no native compiler/source-build/full GCC correctness is claimed.

## Impact

- Files: `bootstrap/diag-gcc40-c-parse-boundary.ncl`, `bootstrap/evidence/gcc-4.0-native-cc1-build-frontier.json`, `src/bootstrap_parity.rs`, `openspec/specs/bootstrap/spec.md`.
- APIs: none.
- Verification: diagnostic eval/build smoke, JSON parse, focused parity tests, CLI parity report confirming `gcc.4.0 partial`, source-pin/blocker self-tests, OpenSpec strict validation, and `git diff --check`.
