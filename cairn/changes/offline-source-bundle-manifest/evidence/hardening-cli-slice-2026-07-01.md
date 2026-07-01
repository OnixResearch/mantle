# Hardening and CLI slice — Offline source bundle manifest

Date: 2026-07-01

## Implemented in this slice

- Hardened `mantle-source-bundle-v1` validation so parsed bundles reject mismatched root order, malformed record order, store-prefix drift for store-bound records, non-claim text drift, tampered file payload metadata, unsafe file entries, symlink payload drift, payload byte-count mismatch, and case-folded path collisions before source state can be persisted.
- Tightened generic adapter metadata validation to fail closed when lock identity, offline control, or generated-source boundary facts are absent.
- Added a positive non-Cargo `package-mirror` adapter fixture at the pure core boundary.
- Added negative pure tests for incomplete adapter metadata, malformed roots/order, tampered payload metadata, and store-prefix mismatch.
- Added CLI tests covering no-mutate `source bundle plan`, no-mutate `source bundle list`, import/list/verify round trip over imported source state, and tampered bundle rejection without persisted records.

## Baseline evidence before this slice

```text
$ cargo test -p mantle source_bundle::
sh: line 1: cargo: command not found

$ nix develop -c cargo test -p mantle --bin mantle source_bundle -- --list && nix develop -c cargo test -p mantle --bin mantle source_bundle
...
test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 1023 filtered out; finished in 0.00s
```

## Focused validation after this slice

```text
$ nix develop -c cargo test -p mantle --bin mantle source_bundle
...
test result: ok. 22 passed; 0 failed; 0 ignored; 0 measured; 1023 filtered out; finished in 0.00s

$ nix develop -c cargo test -p mantle --test source_bundle_cli
running 2 tests
test source_bundle_cli_rejects_tampered_bundle_without_persisting_records ... ok
test source_bundle_cli_round_trips_imported_source_state ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
```

Formatting and Cairn lifecycle checks:

```text
$ nix develop -c cargo fmt -p mantle --check
completed successfully

$ nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
valid: true

$ nix run path:/home/brittonr/git/cairn#cairn -- gate proposal offline-source-bundle-manifest --root .
verdict: PASS

$ nix run path:/home/brittonr/git/cairn#cairn -- gate design offline-source-bundle-manifest --root .
verdict: PASS

$ nix run path:/home/brittonr/git/cairn#cairn -- gate tasks offline-source-bundle-manifest --root .
verdict: PASS
```

## Remaining gap before archive

This remains a hardening slice, not a complete drain. Non-local payload acquisition, revision-checked VCS snapshots, Cargo plus broader non-Cargo adapter fixtures, bootstrap/provider/toolchain/proof fixtures, remote-builder input preparation, and broader missing/stale/unsupported/untrusted offline-preflight coverage remain before `offline-source-bundle-manifest` can be archived.
