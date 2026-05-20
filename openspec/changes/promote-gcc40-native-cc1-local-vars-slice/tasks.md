## Change Status

Incomplete: scaffold is present; implementation and archive remain pending.

## Phase 1: Spec and receipt contract

- [x] [serial] Create the OpenSpec scaffold for the v3 native `cc1` local-variable assignment slice.
- [x] [depends:scaffold] Validate the scaffold with strict OpenSpec checks and commit it.
- [ ] [depends:contract] Finalize the v3 receipt schema fields, selected-slice name, and preserved-regression requirements.
- [ ] [depends:contract] Update the bootstrap spec delta for accepted, drift, and no-overclaim scenarios.

## Phase 2: Implementation

- [ ] [depends:contract] Add the bounded no-TinyCC-delegation local-variable assignment input path to installed `cc1`.
- [ ] [depends:implementation] Update receipts, native boundary evidence, placeholder inventory, and parity validation.
- [ ] [depends:implementation] Add positive and negative parity regressions for valid evidence, stale schema, missing local marker, digest drift, TinyCC delegation, and missing prior regressions.

## Phase 3: Verification and archive

- [ ] [depends:implementation] Run focused GCC 4.0 parity tests, parity snapshot, OpenSpec strict validation, and `git diff --check`.
- [ ] [depends:verification] Archive the change, repair any canonical spec drift, rerun final validation, commit, and push.

## Verification Coverage

- Positive: valid v3 receipt keeps `gcc.4.0` evidence-backed `partial` with local-variable slice evidence.
- Negative: stale schema, missing marker, digest drift, TinyCC delegation marker, omitted arithmetic regression, omitted logical regression, or parity overclaim fails closed.
- Non-claim: live-bootstrap, Guix, and StageX remain blocked until broader GCC 4.0 native correctness exists.
