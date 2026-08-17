# CLI preflight failure coverage — 2026-07-01

Task-ID: implementation-slice
Covers: source_transports.offline_build_preflight, source_transports.source_bundle_import_verify

## Question

Do source-bundle CLI tests cover fail-closed source-state diagnostics beyond the happy import/verify path?

## Inspected evidence

- `tests/source_bundle_cli.rs` now covers stale imported state by changing a persisted record identity while keeping its content digest valid, then verifying `ready_class = stale` and one stale record.
- It covers unpinned local source state by exporting/importing a local `file://` fetcher root without `--pin`, then proving `source bundle preflight` exits non-zero with `ready_class = unpinned` before build execution.
- It covers remote/non-local acquisition policy by proving `source bundle preflight --build-root examples/fetch-crate-crc64.ncl` exits non-zero with `ready_class = network-required` instead of fetching or building.
- Existing CLI tests continue to cover no-mutate plan/list, missing-before-import verify, import/list/verify round trip, and tampered-bundle rejection without persisted records.

## Validation transcript

```text
$ nix develop -c cargo test -p mantle --test source_bundle_cli -- --nocapture

running 5 tests
test source_bundle_cli_rejects_tampered_bundle_without_persisting_records ... ok
test source_bundle_cli_reports_stale_imported_state ... ok
test source_bundle_cli_round_trips_imported_source_state ... ok
test source_bundle_cli_preflight_reports_network_required_before_build ... ok
test source_bundle_cli_preflight_reports_unpinned_imported_state ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.14s
```

```text
$ nix develop -c cargo fmt --check && nix develop -c cargo check -p mantle && nix develop -c cargo test -p mantle --bin mantle source_bundle && nix develop -c cargo test -p mantle --test source_bundle_cli && nix run path:/home/brittonr/git/cairn#cairn -- validate --root .

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.12s

{
  "change_issues": [],
  "changes": 7,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 18,
  "valid": true
}
```

## Decision

This removes the broad CLI failure-fixture gap for missing/stale/network-required/unpinned source readiness at the current source-state boundary. It still does not implement real package-manager/provider/toolchain acquisition shells.

## Next action

Decide whether the remaining package-manager/provider/toolchain/proof inputs should stay pure fixtures for this change or receive real adapter shell support before archiving.
