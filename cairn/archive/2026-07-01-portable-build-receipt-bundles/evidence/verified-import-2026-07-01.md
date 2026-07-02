# Verified import and trust replay slice — Portable build receipt bundles

Date: 2026-07-01

## Implemented in this slice

- Added explicit trust replay context for receipt bundle output verification: policy hash, verification timestamp, optional revocation-list ref, and revoked public key digests.
- Verification now rejects expired trust windows, revoked public key digests, revocation-ref mismatches, malformed trust snapshots, wrong store prefixes, stale local PathInfo digests, and missing output refs before reporting a verified output match.
- `mantle receipt bundle import` now requires `--output` and `--policy-hash`, verifies the bundle against local output facts/trust context, then imports evidence only after verification succeeds.
- Imported receipt bundles now merge semantic graph nodes/edges derived from output/source/action/sandbox/trust records and `semantic-graph-edge` records, rejecting graph node/edge conflicts without persisting the bundle.
- Added proof that a verified imported bundle can answer `mantle why` through imported semantic graph evidence, while conflicting graph imports fail closed.

## Validation transcript


### focused-tests

```text
$ export PATH="/nix/store/97vplpbajnr7x03fqh9biz5v6960sv22-clang-wrapper-21.1.8/bin:/nix/store/1sw8whfl5gfblp6r9qdkiw1b4j9fgwar-mold-2.40.4/bin:/nix/store/rvp7qlpf5jqvdckjy1afjb6aha6j8dxg-pkg-config-wrapper-0.29.2/bin:/run/current-system/sw/bin:$PATH"; export CC=clang; export CARGO_TARGET_DIR=/home/brittonr/.cargo-target; export RUSTC=/home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/rustc; export PKG_CONFIG_PATH=/nix/store/1l5jgzy26hkjz1y3apn1051asvn42sfn-openssl-3.6.1-dev/lib/pkgconfig; mkdir -p target/tmp; TMPDIR=$PWD/target/tmp /home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/cargo test -p mantle --bin mantle portable_receipt::tests
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Blocking waiting for file lock on build directory
    Finished `test` profile [unoptimized + debuginfo] target(s) in 35.72s
     Running unittests src/main.rs (/home/brittonr/.cargo-target/debug/deps/mantle-a789c34159ead64f)

running 18 tests
test portable_receipt::tests::metadata_path_traversal_is_rejected ... ok
test portable_receipt::tests::malformed_record_spec_is_rejected ... ok
test portable_receipt::tests::duplicate_receipt_records_fail_closed ... ok
test portable_receipt::tests::complete_bundle_supports_strong_claim_classification ... ok
test portable_receipt::tests::partial_bundle_is_diagnostic_for_strong_claim ... ok
test portable_receipt::tests::missing_output_state_does_not_fabricate_records ... ok
test portable_receipt::tests::graph_and_source_state_records_are_gathered_without_placeholders ... ok
test portable_receipt::tests::receipt_import_is_idempotent_and_conflict_checked ... ok
test portable_receipt::tests::missing_output_ref_rejects_output_verification ... ok
test portable_receipt::tests::policy_hash_mismatch_rejects_output_verification ... ok
test portable_receipt::tests::revoked_public_key_rejects_output_verification ... ok
test portable_receipt::tests::expired_trust_snapshot_rejects_output_verification ... ok
test portable_receipt::tests::wrong_store_prefix_rejects_output_verification ... ok
test portable_receipt::tests::pathinfo_and_attestation_state_records_are_gathered_for_output ... ok
test portable_receipt::tests::matching_local_output_verifies_against_policy_and_pathinfo ... ok
test portable_receipt::tests::stale_local_output_digest_rejects_output_verification ... ok
test portable_receipt::tests::conflicting_graph_import_fails_without_persisting_bundle ... ok
test portable_receipt::tests::verified_import_persists_graph_evidence_for_why ... ok

test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 1053 filtered out; finished in 0.04s


status: ok
```

### rustfmt-check

```text
$ mkdir -p target/tmp; TMPDIR=$PWD/target/tmp /home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/rustfmt --check src/portable_receipt.rs src/main.rs

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
