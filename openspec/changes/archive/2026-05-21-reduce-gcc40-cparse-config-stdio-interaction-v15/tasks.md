## Phase 1: Scaffold

- [x] [serial] Inspect v14 receipt, diagnostic derivation, and parity validation seam.
- [x] [serial] Create and validate this OpenSpec change.

## Phase 2: Implementation

- [x] [serial] Add compact v15 `config.h` + `<stdio.h>` interaction probes to the diagnostic derivation.
- [x] [depends:diagnostic] Run the smallest diagnostic/eval/build probe and record observed v15 values.
- [x] [serial] Update the c-parse frontier receipt, parity validation/tests, and canonical bootstrap spec while keeping `gcc.4.0` partial.

## Phase 3: Verification and Landing

- [x] [depends:evidence] Run JSON, focused parity tests, CLI parity report, blocker/source-pin self-tests, OpenSpec strict validation, and `git diff --check`.
- [x] [serial] Archive the completed OpenSpec, repair any archive spec drift, commit/push, and update the workflow reference if a reusable pitfall is found.
