# Archive output facts slice — Portable build receipt bundles

Date: 2026-07-01

## Implemented in this slice

- Extended `ArchiveListedPath` with references, signatures, and deriver fields so archive list metadata carries enough PathInfo identity facts to recompute receipt output-ref digests.
- Added archive-output fact verification for receipt bundles: `verify` and verified `import` can consume a store archive via `--archive <path>` instead of requiring local PathInfo state.
- Added positive coverage proving archive output facts verify a bundle without local PathInfo.
- Added negative coverage proving stale archive output facts reject verification.
- Re-ran `crunch-store` archive tests to cover the archive list/report shape change.

## Validation transcript


### receipt-tests

```text
$ export PATH="/nix/store/97vplpbajnr7x03fqh9biz5v6960sv22-clang-wrapper-21.1.8/bin:/nix/store/1sw8whfl5gfblp6r9qdkiw1b4j9fgwar-mold-2.40.4/bin:/nix/store/rvp7qlpf5jqvdckjy1afjb6aha6j8dxg-pkg-config-wrapper-0.29.2/bin:/run/current-system/sw/bin:$PATH"; export CC=clang; export CARGO_TARGET_DIR=/home/brittonr/.cargo-target; export RUSTC=/home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/rustc; export PKG_CONFIG_PATH=/nix/store/1l5jgzy26hkjz1y3apn1051asvn42sfn-openssl-3.6.1-dev/lib/pkgconfig; mkdir -p target/tmp; TMPDIR=$PWD/target/tmp /home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/cargo test -p mantle --bin mantle portable_receipt::tests
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Blocking waiting for file lock on build directory
    Finished `test` profile [unoptimized + debuginfo] target(s) in 19.31s
     Running unittests src/main.rs (/home/brittonr/.cargo-target/debug/deps/mantle-a789c34159ead64f)

running 20 tests
test portable_receipt::tests::malformed_record_spec_is_rejected ... ok
test portable_receipt::tests::duplicate_receipt_records_fail_closed ... ok
test portable_receipt::tests::metadata_path_traversal_is_rejected ... ok
test portable_receipt::tests::complete_bundle_supports_strong_claim_classification ... ok
test portable_receipt::tests::partial_bundle_is_diagnostic_for_strong_claim ... ok
test portable_receipt::tests::missing_output_state_does_not_fabricate_records ... ok
test portable_receipt::tests::graph_and_source_state_records_are_gathered_without_placeholders ... ok
test portable_receipt::tests::receipt_import_is_idempotent_and_conflict_checked ... ok
test portable_receipt::tests::missing_output_ref_rejects_output_verification ... ok
test portable_receipt::tests::pathinfo_and_attestation_state_records_are_gathered_for_output ... ok
test portable_receipt::tests::stale_archive_output_fact_rejects_verification ... ok
test portable_receipt::tests::archive_output_facts_verify_without_local_pathinfo ... ok
test portable_receipt::tests::wrong_store_prefix_rejects_output_verification ... ok
test portable_receipt::tests::expired_trust_snapshot_rejects_output_verification ... ok
test portable_receipt::tests::revoked_public_key_rejects_output_verification ... ok
test portable_receipt::tests::policy_hash_mismatch_rejects_output_verification ... ok
test portable_receipt::tests::matching_local_output_verifies_against_policy_and_pathinfo ... ok
test portable_receipt::tests::stale_local_output_digest_rejects_output_verification ... ok
test portable_receipt::tests::conflicting_graph_import_fails_without_persisting_bundle ... ok
test portable_receipt::tests::verified_import_persists_graph_evidence_for_why ... ok

test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 1053 filtered out; finished in 0.04s


status: ok
```

### archive-tests

```text
$ export PATH="/nix/store/97vplpbajnr7x03fqh9biz5v6960sv22-clang-wrapper-21.1.8/bin:/nix/store/1sw8whfl5gfblp6r9qdkiw1b4j9fgwar-mold-2.40.4/bin:/nix/store/rvp7qlpf5jqvdckjy1afjb6aha6j8dxg-pkg-config-wrapper-0.29.2/bin:/run/current-system/sw/bin:$PATH"; export CC=clang; export CARGO_TARGET_DIR=/home/brittonr/.cargo-target; export RUSTC=/home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/rustc; export PKG_CONFIG_PATH=/nix/store/1l5jgzy26hkjz1y3apn1051asvn42sfn-openssl-3.6.1-dev/lib/pkgconfig; mkdir -p target/tmp; TMPDIR=$PWD/target/tmp /home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/cargo test -p crunch-store archive
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.20s
     Running unittests src/lib.rs (/home/brittonr/.cargo-target/debug/deps/crunch_store-b2b617915938de54)

running 16 tests
test archive::tests::import_action_core_rejects_untrusted_and_skips_present ... ok
test archive::tests::archive_list_rejects_bad_magic ... ok
test archive::tests::archive_list_rejects_header_end_count_mismatch ... ok
test archive::tests::pathinfo_fixture_service_is_bounded ... ok
test archive::tests::archive_export_list_round_trip_preserves_metadata_before_payload ... ok
test archive::tests::archive_export_refuses_unsigned_without_escape_hatch ... ok
test archive::tests::archive_list_drains_non_seekable_payloads_in_bounded_chunks ... ok
test archive::tests::export_closure_includes_references_deterministically ... ok
test archive::tests::missing_closure_reference_fails_export_plan ... ok
test archive::tests::archive_import_rejects_unsupported_ca_metadata_without_persisting ... ok
test archive::tests::archive_import_rejects_store_prefix_mismatch_before_persisting ... ok
test archive::tests::archive_import_rejects_conflicting_local_pathinfo ... ok
test archive::tests::archive_import_rejects_tampered_payload_without_persisting ... ok
test archive::tests::archive_import_rejects_untrusted_signature_without_persisting ... ok
test archive::tests::archive_import_rejects_truncated_payload_without_persisting ... ok
test archive::tests::archive_import_round_trip_and_skip_existing_are_idempotent ... ok

test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 141 filtered out; finished in 0.09s


status: ok
```

### rustfmt-check

```text
$ mkdir -p target/tmp; TMPDIR=$PWD/target/tmp /home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/rustfmt --check src/portable_receipt.rs src/main.rs crates/crunch-store/src/archive.rs

status: ok
```

### diff-check

```text
$ git diff --check

status: ok
```

### cairn-validate

```text
$ mkdir -p target/tmp; TMPDIR=$PWD/target/tmp nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
{
  "change_issues": [],
  "changes": 2,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 13,
  "valid": true
}

status: ok
```

### cairn-gate-proposal

```text
$ mkdir -p target/tmp; TMPDIR=$PWD/target/tmp nix run path:/home/brittonr/git/cairn#cairn -- gate proposal portable-build-receipt-bundles --root .
{
  "change": "portable-build-receipt-bundles",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "51f9c3f4d79d87bbc2d9e7c7d98c3170ea1bec79fde7ad93faabd28ff33a3355",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "81b06862ca3481c98586f0d73f1724c25368ac3ac1f0764c09c9531e2552b9ed",
  "stage": "proposal",
  "valid": true,
  "verdict": "PASS"
}

status: ok
```

### cairn-gate-design

```text
$ mkdir -p target/tmp; TMPDIR=$PWD/target/tmp nix run path:/home/brittonr/git/cairn#cairn -- gate design portable-build-receipt-bundles --root .
{
  "change": "portable-build-receipt-bundles",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "27bbbacf73072e01f1e6d3157131a49e9237e14568475f44575404c368a593a7",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "657cc7143a199baeafc2c46b2e40f086a6d363cd91ad265f5d6baea64d4f61d4",
  "stage": "design",
  "valid": true,
  "verdict": "PASS"
}

status: ok
```

### cairn-gate-tasks

```text
$ mkdir -p target/tmp; TMPDIR=$PWD/target/tmp nix run path:/home/brittonr/git/cairn#cairn -- gate tasks portable-build-receipt-bundles --root .
{
  "change": "portable-build-receipt-bundles",
  "evidence_summary": {
    "docs_only": 0,
    "fixture": 0,
    "formal": 0,
    "mode": "advisory",
    "model": 0,
    "probe": 0,
    "property": 0
  },
  "input_hash": "dad0e2f32502a2cc70a3be885a2e3e0b5de8a93566cc940efb2a9e6ba9f5991d",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "policy_hash": "2ba17ace71e36a2d8f03f0dc5eaa805a6008e970f2e56a53ff72b891601de119",
  "receipt_hash": "b168946440ea3f9cef1c31c7f3f1ad7bab04002b3bbf193f2a90584575df8486",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}

status: ok
```
