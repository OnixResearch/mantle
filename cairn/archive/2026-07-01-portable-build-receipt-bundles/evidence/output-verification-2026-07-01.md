# Local output verification slice — Portable build receipt bundles

Date: 2026-07-01

## Implemented in this slice

- Added local output verification for `mantle receipt bundle verify --from <bundle> --output <logical-path-or-store-basename> --policy-hash <expected>`.
- Verification now fails closed when an output selector is supplied without an expected policy hash.
- Verification checks the bundle store prefix against the active local store prefix before output matching.
- Verification checks the bundle trust-snapshot policy hash against the expected policy hash.
- Verification recomputes local output-ref records from `state_dir/pathinfo.redb` and requires the bundle output-ref digest to match the local PathInfo facts.
- Verification reports matched output identity, record digest, NAR SHA-256, and NAR size.
- Added negative coverage for wrong store prefix, stale local PathInfo digest, missing bundle output-ref, and policy hash mismatch.

## Baseline before change

```text
$ TMPDIR=$PWD/target/tmp CARGO_TARGET_DIR=/home/brittonr/.cargo-target RUSTC=/home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/rustc /home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/cargo test -p mantle --bin mantle portable_receipt::tests
running 9 tests
test portable_receipt::tests::malformed_record_spec_is_rejected ... ok
test portable_receipt::tests::metadata_path_traversal_is_rejected ... ok
test portable_receipt::tests::duplicate_receipt_records_fail_closed ... ok
test portable_receipt::tests::missing_output_state_does_not_fabricate_records ... ok
test portable_receipt::tests::complete_bundle_supports_strong_claim_classification ... ok
test portable_receipt::tests::partial_bundle_is_diagnostic_for_strong_claim ... ok
test portable_receipt::tests::graph_and_source_state_records_are_gathered_without_placeholders ... ok
test portable_receipt::tests::receipt_import_is_idempotent_and_conflict_checked ... ok
test portable_receipt::tests::pathinfo_and_attestation_state_records_are_gathered_for_output ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 1053 filtered out; finished in 0.01s
```

## Focused tests after change

```text
$ TMPDIR=$PWD/target/tmp CARGO_TARGET_DIR=/home/brittonr/.cargo-target RUSTC=/home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/rustc /home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/cargo test -p mantle --bin mantle portable_receipt::tests
running 14 tests
test portable_receipt::tests::duplicate_receipt_records_fail_closed ... ok
test portable_receipt::tests::complete_bundle_supports_strong_claim_classification ... ok
test portable_receipt::tests::partial_bundle_is_diagnostic_for_strong_claim ... ok
test portable_receipt::tests::missing_output_state_does_not_fabricate_records ... ok
test portable_receipt::tests::graph_and_source_state_records_are_gathered_without_placeholders ... ok
test portable_receipt::tests::receipt_import_is_idempotent_and_conflict_checked ... ok
test portable_receipt::tests::missing_output_ref_rejects_output_verification ... ok
test portable_receipt::tests::wrong_store_prefix_rejects_output_verification ... ok
test portable_receipt::tests::pathinfo_and_attestation_state_records_are_gathered_for_output ... ok
test portable_receipt::tests::policy_hash_mismatch_rejects_output_verification ... ok
test portable_receipt::tests::matching_local_output_verifies_against_policy_and_pathinfo ... ok
test portable_receipt::tests::stale_local_output_digest_rejects_output_verification ... ok

test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 1053 filtered out; finished in 0.04s
```

## Formatting and lifecycle gates

```text
$ TMPDIR=$PWD/target/tmp /home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/rustfmt --check src/portable_receipt.rs src/main.rs
status: ok

$ git diff --check
status: ok

$ TMPDIR=$PWD/target/tmp nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
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

$ TMPDIR=$PWD/target/tmp nix run path:/home/brittonr/git/cairn#cairn -- gate proposal portable-build-receipt-bundles --root .
{
  "issues": [],
  "stage": "proposal",
  "valid": true,
  "verdict": "PASS"
}

$ TMPDIR=$PWD/target/tmp nix run path:/home/brittonr/git/cairn#cairn -- gate design portable-build-receipt-bundles --root .
{
  "issues": [],
  "stage": "design",
  "valid": true,
  "verdict": "PASS"
}

$ TMPDIR=$PWD/target/tmp nix run path:/home/brittonr/git/cairn#cairn -- gate tasks portable-build-receipt-bundles --root .
{
  "issues": [],
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```

## Remaining gap before archive

This still does not complete portable receipt bundles. Verify/import still needs signature trust roots, revocation and expiration checks, archive-provided output facts, source-bundle ref verification beyond imported source-state matching, persisted graph imports, conflict diagnostics, CLI integration tests, and `mantle why` proof over imported remote/offline graph evidence.
