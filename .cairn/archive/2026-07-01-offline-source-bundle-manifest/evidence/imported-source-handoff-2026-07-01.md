# Imported source handoff slice — Offline source bundle manifest

Date: 2026-07-01

## Implemented in this slice

- Added an explicit no-network handoff for remote fixed URL and VCS source records: `source bundle export --build-root` now checks Mantle source state for an already-imported materialized record whose kind, identity, adapter, store prefix, and metadata match the planned source record.
- If matching imported source state contains payload files, export reuses that local materialized payload in the outgoing bundle.
- If no matching imported payload exists, export keeps the previous fail-closed behavior for non-local URLs instead of fetching network material.
- Tightened source-state matching so adapter metadata must match before an imported record can satisfy preflight/export materialization.

## Focused validation

```text
$ nix develop -c cargo check -p mantle
completed successfully

$ nix develop -c cargo test -p mantle --bin mantle source_bundle
...
test source_bundle::tests::source_bundle_export_uses_imported_state_for_remote_fetcher_payload ... ok
test source_bundle::tests::source_bundle_export_rejects_imported_remote_payload_with_wrong_metadata ... ok
...
test result: ok. 25 passed; 0 failed; 0 ignored; 0 measured; 1023 filtered out; finished in 0.00s

$ nix develop -c cargo test -p mantle --test source_bundle_cli
running 2 tests
test source_bundle_cli_rejects_tampered_bundle_without_persisting_records ... ok
test source_bundle_cli_round_trips_imported_source_state ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
```

## Remaining gap before archive

This slice still does not fetch non-local payloads, prove VCS revisions while snapshotting, or prepare remote-builder uploads from imported source state. It only defines the local imported-source handoff and preserves fail-closed behavior when matching source state is absent.
