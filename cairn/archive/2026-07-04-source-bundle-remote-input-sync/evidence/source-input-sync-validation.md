# Source-bundle remote input sync validation

Task-ID: source-bundle-remote-input-sync
Covers: r[remote_builds.source_bundle_input_sync]
Date: 2026-07-04

## Remote source input upload

`pueue task 364`:

```text
Command: nix develop -c cargo test -p mantle --bin mantle source_input_upload -- --nocapture

running 4 tests
test remote_build::tests::source_input_upload_without_store_or_source_state_fails_without_network_fetch ... ok
test remote_build::tests::source_input_upload_rejects_tampered_artifact_payload ... ok
test remote_build::tests::source_input_upload_artifact_materializes_remote_input_bytes ... ok
test remote_build::tests::source_input_upload_reuses_imported_source_state_when_store_path_is_absent ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 1177 filtered out; finished in 0.02s
```

## Upload privacy and readiness policy

`pueue task 363`:

```text
Command: nix develop -c cargo test -p mantle --bin mantle source_input_upload -- --nocapture && nix develop -c cargo test -p mantle --bin mantle source_upload -- --nocapture

running 3 tests
test remote_build::tests::source_upload_readiness_rejects_stale_or_unsupported_records ... ok
test remote_build::tests::source_upload_privacy_policy_rejects_secret_class_and_quota ... ok
test remote_build::tests::source_upload_privacy_summary_counts_classes_before_transfer ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 1178 filtered out; finished in 0.00s
```

## Source-bundle state rail

`pueue task 365`:

```text
Command: nix develop -c cargo test -p mantle --bin mantle source_bundle -- --nocapture

test source_bundle::tests::source_offline_preflight_reports_remote_network_requirement ... ok
test source_bundle::tests::source_offline_preflight_reports_missing_local_source_state ... ok
test source_bundle::tests::source_offline_preflight_reports_unsupported_adapter ... ok
test source_bundle::tests::source_bundle_export_rejects_imported_remote_payload_with_wrong_metadata ... ok
test source_bundle::tests::source_bundle_export_materializes_local_vcs_snapshot_without_dot_git ... ok
test source_bundle::tests::source_bundle_verify_reports_missing_state ... ok
test source_bundle::tests::source_offline_preflight_rejects_unpinned_imported_source ... ok
test source_bundle::tests::source_offline_preflight_accepts_pinned_imported_file_payload ... ok
test source_bundle::tests::source_offline_preflight_reports_untrusted_adapter ... ok
test source_bundle::tests::source_bundle_export_uses_imported_state_for_remote_fetcher_payload ... ok
test source_bundle::tests::source_bundle_accepts_language_neutral_package_adapters ... ok
test source_bundle::tests::source_bundle_import_and_verify_are_idempotent ... ok

test result: ok. 31 passed; 0 failed; 0 ignored; 0 measured; 1150 filtered out; finished in 0.01s
```

## Cairn gates

`pueue task 366`:

```text
Command: nix run path:/home/brittonr/git/cairn#cairn -- gate proposal source-bundle-remote-input-sync --root /home/brittonr/git/mantle && nix run path:/home/brittonr/git/cairn#cairn -- gate design source-bundle-remote-input-sync --root /home/brittonr/git/mantle && nix run path:/home/brittonr/git/cairn#cairn -- gate tasks source-bundle-remote-input-sync --root /home/brittonr/git/mantle
...
"stage": "tasks",
"valid": true,
"verdict": "PASS"
```
