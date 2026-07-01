# Fixture slice — Offline source bundle manifest

Date: 2026-07-01

## Implemented in this slice

- Added pure package-manager mirror fixture coverage for both Cargo adapter metadata and a non-Cargo npm adapter, using the same generic `SourceRecordKind::PackageMirror` machinery.
- Added pure fixture coverage for bootstrap archive, provider manifest, toolchain/source-root, and proof-input source record kinds.
- Confirmed provider/toolchain source records bind the configured logical store prefix, while bootstrap/proof records stay ordinary source records.

## Focused validation

```text
$ nix develop -c cargo test -p mantle --bin mantle source_bundle
...
test source_bundle::tests::source_bundle_accepts_language_neutral_package_adapters ... ok
test source_bundle::tests::source_bundle_covers_bootstrap_provider_toolchain_and_proof_records ... ok
...
test result: ok. 23 passed; 0 failed; 0 ignored; 0 measured; 1023 filtered out; finished in 0.00s

$ nix develop -c cargo test -p mantle --test source_bundle_cli
running 2 tests
test source_bundle_cli_rejects_tampered_bundle_without_persisting_records ... ok
test source_bundle_cli_round_trips_imported_source_state ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
```

## Remaining gap before archive

This fixture slice does not implement non-local payload acquisition, revision-checked VCS snapshots, remote-builder input preparation, or broader CLI preflight fixtures for stale/unsupported/untrusted source state.
