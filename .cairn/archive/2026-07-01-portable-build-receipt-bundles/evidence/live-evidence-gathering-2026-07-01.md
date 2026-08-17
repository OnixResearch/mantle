# Live evidence gathering slice — Portable build receipt bundles

Date: 2026-07-01

## Implemented in this slice

- Added `mantle receipt bundle export --output <logical-path-or-store-basename>` state gathering.
- Gathered local PathInfo output records from `state_dir/pathinfo.redb` with deterministic BLAKE3 record digests over PathInfo identity facts.
- Gathered artifact and runtime-closure attestation sidecar records when their deterministic sidecar files exist.
- Gathered semantic graph edge records and graph-derived source/action/sandbox/trust-basis records when `state_dir/semantic-graph.json` has matching output graph material.
- Bound imported source-bundle state records to graph source nodes when `state_dir/source-bundles/records/*.json` has matching identities.
- Added record metadata validation and fail-closed path-traversal rejection.
- Kept missing output state non-fabricating: absent PathInfo/graph/source state produces no placeholder records and cannot build a strong evidence bundle by itself.

## Baseline note

The first baseline command using `/tmp/mantle-target-receipts` could not establish a test baseline because `/tmp` was full/quota-exceeded on this host:

```text
$ CARGO_TARGET_DIR=/tmp/mantle-target-receipts cargo test -p mantle --bin mantle portable_receipt::tests
sh: line 1: cargo: command not found

$ CARGO_TARGET_DIR=/tmp/mantle-target-receipts nix develop -c cargo test -p mantle --bin mantle portable_receipt::tests
rustc-LLVM ERROR: IO failure on output stream: Disk quota exceeded
error: failed to write `/tmp/mantle-target-receipts/debug/.fingerprint/nickel-lang-core-99cc93767a357575/invoked.timestamp`
```

Validation below uses repo-local `TMPDIR=$PWD/target/tmp` and shared `CARGO_TARGET_DIR=/home/brittonr/.cargo-target`.

## Focused tests

```text
$ TMPDIR=$PWD/target/tmp CARGO_TARGET_DIR=/home/brittonr/.cargo-target RUSTC=/home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/rustc /home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/cargo test -p mantle --bin mantle portable_receipt::tests
running 9 tests
test portable_receipt::tests::malformed_record_spec_is_rejected ... ok
test portable_receipt::tests::metadata_path_traversal_is_rejected ... ok
test portable_receipt::tests::duplicate_receipt_records_fail_closed ... ok
test portable_receipt::tests::missing_output_state_does_not_fabricate_records ... ok
test portable_receipt::tests::partial_bundle_is_diagnostic_for_strong_claim ... ok
test portable_receipt::tests::complete_bundle_supports_strong_claim_classification ... ok
test portable_receipt::tests::graph_and_source_state_records_are_gathered_without_placeholders ... ok
test portable_receipt::tests::receipt_import_is_idempotent_and_conflict_checked ... ok
test portable_receipt::tests::pathinfo_and_attestation_state_records_are_gathered_for_output ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 1053 filtered out; finished in 0.01s
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

This is still not a complete drain. Remaining work includes live action receipt/reference-scan records beyond graph summaries, verify/import against local or archive-provided output facts and trust policy, signature/revocation/expiration checks, semantic graph persistence from verified imports, CLI conflict-diagnostic tests, and `mantle why` integration proof.
