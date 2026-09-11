# Evidence: release observation binding (2026-09-11)

Task-ID: mantle.release_provenance.source_observation_binding
Covers: source_observation_binding, source_observation_signature_boundary

## What landed

- `crunch-release-core`: `SourceObservationBinding` DTO and
  `SourceAcquisition.source_observation` (optional, absent by default), plus
  pure validation reached through `validate_source_acquisition`.
- Rejections: unsupported schema, unsupported encoding version, payload
  digest that does not match the exact release source bytes, an observation
  digest that is a re-labeled content digest, observation kind that does not
  match the acquisition class, missing or drifting snapshot profile, and
  malformed digests.
- Shell: `ReleaseBundleCreateRequest.source_observation`, attachment in
  `build_source_acquisition`, and the operator surface
  `mantle release create --source-observation <binding.json>`; no new signer
  role and no change to the existing release-attestation signature path.

## Rails

- `cargo test -p crunch-release-core`: `241 passed; 0 failed` plus one
  integration test.
- `cargo test -p mantle --test release_cli`: `149 passed; 0 failed`.
- Tiger Style consumer check: exit 0.
- First-party Clippy (`./scripts/check-first-party-clippy.sh`): fails on
  pre-existing debt outside this diff — `tests/store_gc_cli.rs` unused
  `meta` and `src/remote_nominal.rs` unused `as_str` (neither file is
  touched by this branch; recorded as pre-existing, not repaired here).

## Open

- I12 machine-contract and Nickel review-contract updates for the new
  binding field.
- I3/I4 dependency-gated tasks, I9/I10 adapter and staged-publication shells,
  V1/V6/V7/V8 corpus and full-rail tasks.
