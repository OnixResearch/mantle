## Phase 1: Establish the failure

- [x] [serial] I1 Record the baseline gate failure and distinguish historical receipt identity from current source agreement. r[mantle.artifact_auth_adoption.live_validation_scope]
  - Evidence: `evidence/baseline.md` records the targeted Nix failure and all six observed BLAKE3 values.

## Phase 2: Repair scoped validation

- [x] [serial] I2 Preserve the historical cutover receipt and replace the live whole-file flake digest comparison with exact source-declaration validation. r[mantle.artifact_auth_adoption.live_validation_scope]
  - Evidence: `flake.nix` now validates one exact accepted input declaration while the historical `nix.flake_blake3` remains in the unchanged typed receipt.
- [x] [parallel] V1 Add positive current and unrelated-change fixtures plus a negative wrong-revision fixture. r[mantle.artifact_auth_adoption.live_validation_scope]
  - Evidence: pueue task `7419` passed the targeted Nix check after Nix evaluated the current flake, an unrelated-comment copy, and a wrong-revision copy.

## Phase 3: Validate and complete

- [ ] [serial] V2 Run the targeted Nix check, Cairn validation, all three change gates, and Tracey coverage. Record exact output. r[mantle.artifact_auth_adoption.live_validation_scope]
- [ ] [serial] V3 Sync, inspect, archive, and record post-archive validation. r[mantle.artifact_auth_adoption.live_validation_scope]
