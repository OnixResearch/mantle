## Why

v21 recovered the concrete include payload behind the truncated GCC 4.0 `c-parse.o` include-flood: generated `config.h` includes `auto-host.h`, which defines `ssize_t`, and the later `system.h -> stdio.h -> bits/alltypes.h` path also needs the host `ssize_t` typedef. v19/v20 showed outer `config.h` order tweaks do not move the real make frontier.

## What Changes

- Add one bounded v22 diagnostic probe that edits generated `gcc/auto-host.h` directly at the `#define ssize_t` seam before the real `make -C gcc c-parse.o` target.
- Record whether this advances beyond the include-flood frontier or preserves rc=2.
- Update the GCC 4.0 source-frontier receipt and fail-closed parity checks with v22 evidence.

## Out of Scope

- Claiming native GCC 4.0 `cc1` correctness.
- Broad generated-config rewrites or another `config.h` ordering sweep.
- Promoting GCC 4.0 beyond partial/frontier status.

## Impact

- Files: `bootstrap/diag-gcc40-c-parse-boundary.ncl`, `bootstrap/evidence/gcc-4.0-native-cc1-build-frontier.json`, `src/bootstrap_parity.rs`, `openspec/specs/bootstrap/spec.md`.
- Verification: diagnostic eval/build, focused parity tests/report, source-pin audit, blocker inventory, strict OpenSpec validation, whitespace check.