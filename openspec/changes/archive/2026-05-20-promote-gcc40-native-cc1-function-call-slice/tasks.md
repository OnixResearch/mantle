## Change Status

Complete: implementation, verification, archive, and final commit preparation passed.

## Phase 1: Spec and receipt contract

- [x] [serial] Create the OpenSpec scaffold for the v4 native `cc1` helper-call slice.
- [x] [depends:scaffold] Validate the scaffold with strict OpenSpec checks.
- [x] [depends:contract] Finalize the v4 receipt schema fields, selected-slice name, and preserved-regression requirements.
- [x] [depends:contract] Update the bootstrap spec delta for accepted, drift, and no-overclaim scenarios.

## Phase 2: Implementation

- [x] [depends:contract] Add the bounded no-TinyCC-delegation helper-call input path to installed `cc1`.
- [x] [depends:implementation] Update receipts, native boundary evidence, placeholder inventory, and parity validation as needed.
- [x] [depends:implementation] Add positive and negative parity regressions for valid evidence, stale schema, missing helper marker, digest drift, TinyCC delegation, and missing prior regressions.

## Phase 3: Verification and archive

- [x] [depends:implementation] Run focused GCC 4.0 parity tests, parity report, blocker/source-pin checks if touched, OpenSpec strict validation, and `git diff --check`.
- [x] [depends:verification] Archive the change, repair any canonical spec drift, rerun final validation, commit, and push.

## Verification Coverage

- Positive: valid v4 receipt keeps `gcc.4.0` evidence-backed `partial` with helper-call slice evidence.
- Negative: stale schema, missing marker, digest drift, TinyCC delegation marker, omitted arithmetic/logical/local-vars regression, or parity overclaim fails closed.
- Non-claim: live-bootstrap, Guix, and StageX remain blocked until broader GCC 4.0 native correctness exists.
