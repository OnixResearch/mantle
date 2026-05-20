## Phase 1: Spec and receipt contract

- [x] [serial] Add the v2 native `cc1` logical/control-flow receipt contract and stale-v1 fail-closed expectations.
- [x] [depends:contract] Update the bootstrap spec delta for accepted, drift, and no-overclaim scenarios.

## Phase 2: Implementation

- [x] [depends:contract] Add the bounded no-TinyCC-delegation logical/control-flow input path to installed `cc1`.
- [x] [depends:implementation] Update receipts, native boundary evidence, placeholder inventory, and parity validation.

## Phase 3: Verification and archive

- [x] [depends:implementation] Run focused GCC 4.0 parity tests, parity snapshot, OpenSpec strict validation, and `git diff --check`.
- [x] [depends:verification] Archive the change, repair any canonical spec drift, rerun final validation, commit, and push.
