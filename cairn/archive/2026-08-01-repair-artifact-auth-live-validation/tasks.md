## Phase 1: Establish the failure

- [x] [serial] I1 Record the baseline gate failure and distinguish historical receipt identity from current source agreement. r[mantle.artifact_auth_adoption.live_validation_scope]
  - Evidence: `evidence/baseline.md` records the targeted Nix failure and all six observed BLAKE3 values.

## Phase 2: Repair scoped validation

- [x] [serial] I2 Preserve the historical cutover receipt and replace the live whole-file flake digest comparison with exact source-declaration validation. r[mantle.artifact_auth_adoption.live_validation_scope]
  - Evidence: `flake.nix` now validates one exact accepted input declaration while the historical `nix.flake_blake3` remains in the unchanged typed receipt.
- [x] [parallel] V1 Add positive current and unrelated-change fixtures plus a negative wrong-revision fixture. r[mantle.artifact_auth_adoption.live_validation_scope]
  - Evidence: pueue task `7419` passed the targeted Nix check after Nix evaluated the current flake, an unrelated-comment copy, and a wrong-revision copy.

## Phase 3: Validate and complete

- [x] [serial] V2 Run the targeted Nix check, Cairn validation, all three change gates, and Tracey coverage. Record exact output. r[mantle.artifact_auth_adoption.live_validation_scope]
  - Evidence: `evidence/validation.md` records focused success, four passing lifecycle commands, scoped Tracey coverage, and the unrelated broad-gate blockers.
- [x] [serial] V3 Sync, inspect, archive, and record post-archive validation. r[mantle.artifact_auth_adoption.live_validation_scope]
  - Evidence: sync plan `2085c9fc4f680b97962fca55e7c1ffa912c90d1a79279f1c5d8429c84b38b656` reported `already_applied`; the canonical requirement preserves all prior artifact-auth requirements. Archive and post-archive output are recorded with this change.
