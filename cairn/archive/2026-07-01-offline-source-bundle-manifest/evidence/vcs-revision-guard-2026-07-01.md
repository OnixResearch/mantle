# VCS revision guard slice — Offline source bundle manifest

Date: 2026-07-01

## Implemented in this slice

- Git/VCS snapshot source records now fail closed unless the fixed fetcher environment carries an explicit `rev` fact.
- The source record metadata still carries the selected `rev`, and imported source-state handoff requires metadata equality before a materialized VCS payload can satisfy export/preflight matching.
- This narrows the VCS snapshot claim to a named selected revision instead of allowing a repository URL alone to stand in for source identity.

## Focused validation

```text
$ nix develop -c cargo check -p mantle
completed successfully

$ nix develop -c cargo test -p mantle --bin mantle source_bundle
...
test source_bundle::tests::source_bundle_derives_vcs_snapshot_from_git_fetcher ... ok
test source_bundle::tests::source_bundle_rejects_vcs_snapshot_without_revision_identity ... ok
...
test result: ok. 26 passed; 0 failed; 0 ignored; 0 measured; 1023 filtered out; finished in 0.00s

$ nix develop -c cargo test -p mantle --test source_bundle_cli
running 2 tests
test source_bundle_cli_rejects_tampered_bundle_without_persisting_records ... ok
test source_bundle_cli_round_trips_imported_source_state ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
```

## Remaining gap before archive

This is a revision-identity guard, not a full VCS proof. Mantle still needs a reviewed revision-checked checkout materialization path before claiming that arbitrary imported VCS payload bytes prove the named revision.
