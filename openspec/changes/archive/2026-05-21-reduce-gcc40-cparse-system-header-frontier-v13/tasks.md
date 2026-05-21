## Phase 1: Scope and diagnostic

- [x] [serial] Inspect current v12 c-parse source-frontier receipt and `system.h` direct include order.
- [x] [serial] Add compact nested `system.h` direct include-prefix probes to the diagnostic derivation.

## Phase 2: Evidence and validation

- [x] [depends:diagnostic] Run the diagnostic or smallest equivalent eval/build probe and record the observed v13 frontier values.
- [x] [serial] Update the c-parse source-frontier receipt and fail-closed parity validation/tests for v13.
- [x] [serial] Update the canonical bootstrap spec under `### Requirement: GCC version ladder`.

## Phase 3: Verification and archive

- [x] [depends:evidence] Run JSON, focused parity tests, CLI parity tests, blocker/source-pin self-tests, parity-report, OpenSpec strict validation, and `git diff --check`.
- [x] [serial] Archive the OpenSpec, inspect/repair canonical spec drift, commit/push, and update the workflow reference.
