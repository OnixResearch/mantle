## Phase 1: Scaffold

- [x] [serial] Inspect v13 receipt and current diagnostic seam.
- [x] [serial] Create and validate this OpenSpec change.

## Phase 2: Implementation

- [x] [serial] Add compact v14 `stdio.h` source-frontier probes to the diagnostic derivation.
- [x] [depends:diagnostic] Run the smallest diagnostic/eval/build probe and record observed v14 values.
- [x] [serial] Update the c-parse frontier receipt, parity validation/tests, and canonical bootstrap spec while keeping `gcc.4.0` partial.

## Phase 3: Verification and Landing

- [x] [depends:evidence] Run JSON, focused parity tests, CLI parity tests, blocker/source-pin self-tests, parity-report, OpenSpec strict validation, and `git diff --check`.
- [x] [serial] Archive the completed OpenSpec, repair any archive spec drift, commit/push, and update the workflow reference.
