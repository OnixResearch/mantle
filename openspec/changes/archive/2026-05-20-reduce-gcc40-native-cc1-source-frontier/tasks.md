## Change Status

Complete: bounded source-frontier reduction evidence is implemented, verified, and ready to archive.

## Phase 1: Spec and planning

- [x] [serial] Create the OpenSpec scaffold for reducing the GCC 4.0 native `cc1` source frontier.
- [x] [depends:scaffold] Validate the scaffold with strict OpenSpec checks and commit it.
- [x] [depends:contract] Choose one bounded source-frontier seam and write down the exact expected probe/receipt contract.

## Phase 2: Implementation

- [x] [depends:contract] Add or update the native `cc1` source-frontier receipt with old frontier, observed new frontier or unchanged blocker, exact source markers, and partial-only parity effect.
- [x] [depends:implementation] Wire fail-closed parity validation for stale/missing frontier evidence and unsupported overclaim fields.
- [x] [depends:implementation] Add positive and negative regressions for valid frontier evidence, marker drift, missing frontier result, and parity overclaim.
- [x] [depends:implementation] Update blocker-inventory suppression checks only if new checked receipt metadata or exact source marker lines move.

## Phase 3: Verification and archive

- [x] [depends:implementation] Run focused GCC 4.0 parity tests and the parity-report CLI.
- [x] [depends:implementation] Run blocker inventory/source-pin checks if source markers or inventory metadata changed.
- [x] [depends:verification] Run `git diff --check`, strict OpenSpec validation for this change, and `openspec validate --all --strict`.
- [x] [depends:verification] Archive the change, repair any cumulative bootstrap spec drift, rerun final validation, commit, and push.

## Verification Coverage

- Positive: valid source-frontier evidence keeps `gcc.4.0` evidence-backed `partial` with a more precise or reduced native `cc1` source-build frontier.
- Negative: marker drift, stale frontier status, missing observed result, unsupported schema/status, or parity overclaim fails closed.
- Non-claim: live-bootstrap, Guix, and StageX remain blocked until full native GCC 4.0 compiler correctness evidence exists.
