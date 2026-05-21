## Phase 1: OpenSpec

- [x] [serial] Validate the v16 proposal/spec/design/tasks before implementation.

## Phase 2: Implementation

- [x] [serial] Add compact `ssize_t` definition/order probes to `bootstrap/diag-gcc40-c-parse-boundary.ncl` without exceeding the diagnostic script-size frontier.
- [x] [serial] Update `bootstrap/evidence/gcc-4.0-native-cc1-build-frontier.json` and `src/bootstrap_parity.rs` to require v16 evidence while keeping `gcc.4.0` partial.
- [x] [serial] Add the v16 scenarios to the base bootstrap spec without dropping prior GCC ladder scenarios.

## Phase 3: Verification and Archive

- [x] [serial] Run focused parity/CLI/source-pin/blocker checks plus `openspec validate --all --strict` and `git diff --check`.
- [x] [serial] Archive the OpenSpec change, commit, push, and confirm a clean synced tree with no active OpenSpec changes.
