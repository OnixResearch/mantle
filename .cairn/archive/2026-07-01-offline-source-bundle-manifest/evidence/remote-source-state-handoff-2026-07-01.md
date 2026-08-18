# Remote source-state handoff — 2026-07-01

Task-ID: implementation-slice
Covers: source_transports.offline_build_preflight, source_transports.source_bundle_import_verify

## Question

Can remote-builder source input upload reuse imported `mantle-source-bundle-v1` state when the declared logical store-path input is absent from the client's physical store path?

## Inspected evidence

- `src/source_bundle.rs` now has a source-state materialization shell that validates an imported record, checks the store prefix plus `store_path` metadata, rejects files below symlink entries before materialization, writes regular files/symlinks into a scratch root, and refuses empty/non-matching records.
- `src/remote_build.rs` now has `populate_remote_input_upload_artifacts_from_store_or_source_state(...)`: the normal local-store path remains preferred, and only `remote-input-source-path-missing` falls back to imported source state before rendering the NAR upload artifact.
- `src/main.rs` now passes the configured `state_dir` into remote input-upload preparation.
- `remote_build::tests::source_input_upload_reuses_imported_source_state_when_store_path_is_absent` seeds `source-bundles/records`, leaves the client store path absent, uploads a NAR artifact, and verifies the remote side materializes the imported payload bytes at the requested store path.

## Validation transcript

```text
$ nix develop -c cargo test -p mantle --bin mantle source_bundle

test result: ok. 27 passed; 0 failed; 0 ignored; 0 measured; 1024 filtered out; finished in 0.00s
```

```text
$ nix develop -c cargo test -p mantle --bin mantle remote_build::tests::source_input_upload

running 3 tests
test remote_build::tests::source_input_upload_rejects_tampered_artifact_payload ... ok
test remote_build::tests::source_input_upload_artifact_materializes_remote_input_bytes ... ok
test remote_build::tests::source_input_upload_reuses_imported_source_state_when_store_path_is_absent ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 1048 filtered out; finished in 0.01s
```

```text
$ nix develop -c cargo fmt --check && nix develop -c cargo check -p mantle && nix develop -c cargo test -p mantle --bin mantle source_bundle && nix develop -c cargo test -p mantle --bin mantle remote_build::tests::source_input_upload && nix develop -c cargo test -p mantle --test source_bundle_cli && nix run path:/home/brittonr/git/cairn#cairn -- validate --root .

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

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

This removes the specific remote-builder input-preparation blocker for already-imported store-path source records. It does not prove acquisition of non-local payloads, revision-checked VCS checkout materialization, or broader adapter acquisition shells.

## Next action

Keep the complete change open and continue with non-local acquisition policy plus revision-checked VCS/source adapter coverage.
