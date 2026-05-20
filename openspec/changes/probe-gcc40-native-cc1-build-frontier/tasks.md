## Change Status

Incomplete: scaffold is present; implementation and archive remain pending.

## Phase 1: Spec and receipt contract

- [x] [serial] Create the OpenSpec scaffold for the native `cc1` build-frontier receipt.
- [x] [depends:scaffold] Validate the scaffold with strict OpenSpec checks and commit it.
- [ ] [depends:contract] Finalize the receipt schema, required derivation markers, source-frontier markers, and retirement condition.
- [ ] [depends:contract] Update the bootstrap spec delta for accepted, drift, and no-overclaim scenarios.

## Phase 2: Implementation

- [ ] [depends:contract] Add the checked native `cc1` build-frontier receipt.
- [ ] [depends:implementation] Wire parity validation for exact marker checks and partial-only semantics.
- [ ] [depends:implementation] Add positive and negative parity regressions for valid evidence, marker drift, missing fields, unsupported parity effect, and parity overclaim.

## Phase 3: Verification and archive

- [ ] [depends:implementation] Run focused GCC 4.0 parity tests, parity snapshot, OpenSpec strict validation, and `git diff --check`.
- [ ] [depends:verification] Archive the change, repair any canonical spec drift, rerun final validation, commit, and push.

## Verification Coverage

- Positive: valid build-frontier receipt keeps `gcc.4.0` evidence-backed `partial` with native source-build frontier evidence.
- Negative: marker drift, stale/missing fields, unsupported parity effect, or parity overclaim fails closed.
- Non-claim: live-bootstrap, Guix, and StageX remain blocked until broader GCC 4.0 native correctness exists.
