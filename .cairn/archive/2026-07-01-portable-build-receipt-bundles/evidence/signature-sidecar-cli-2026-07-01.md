# Signature, sidecar, CLI, and archive-why completion slice — Portable build receipt bundles

Date: 2026-07-01

## Question

Can the remaining `portable-build-receipt-bundles` blockers around real signature verification, embedded attestation sidecar persistence, CLI conflict diagnostics, and archive-backed graph explanation be closed?

## Inspected evidence

- `src/portable_receipt.rs` now verifies PathInfo signatures with trusted `nix_compat::narinfo::VerifyingKey` material, rejects same-name-different-key replay, rejects unsigned outputs when trusted verification is requested, and reports matched trusted public key digests in `ReceiptVerifyReport.signature_matches`.
- `mantle receipt bundle export --trusted-public-key <name:base64>` snapshots trusted public key BLAKE3 digests into the bundle; `verify` and `import` parse the same flag and require the bundle snapshot to match before accepting signatures.
- Export embeds bounded artifact/closure attestation sidecar payloads as base64 with BLAKE3 record digests; bundle validation rejects sidecar digest mismatch; verified import writes sidecar bytes idempotently and rejects conflicting local sidecars before persisting the bundle.
- CLI shell coverage exercises export/list/verify/import with human and JSON paths, trusted signature reporting, embedded sidecar import, and conflict diagnostics.
- Archive-backed import coverage now imports a verified bundle using archive output facts, proves imported semantic graph evidence answers `why`, and separately proves missing graph evidence stays incomplete instead of inventing a producing recipe edge.

## Decision

The blockers recorded in `current-blocker.md` are closed for this change. Remaining action/ref-scan material is represented only when existing records or graph-derived records are available; the implementation still refuses to fabricate placeholders, so incomplete strong-claim bundles stay diagnostic.

## Owner

Mantle receipt/evidence transport.

## Next action

Mark tasks complete, run Cairn gates/validation, sync accepted specs, and archive `portable-build-receipt-bundles` if gates pass.

## Validation transcript

### portable-receipt-focused-tests

```text
$ export PATH="$HOME/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin:/nix/store/97vplpbajnr7x03fqh9biz5v6960sv22-clang-wrapper-21.1.8/bin:/nix/store/1sw8whfl5gfblp6r9qdkiw1b4j9fgwar-mold-2.40.4/bin:/nix/store/rvp7qlpf5jqvdckjy1afjb6aha6j8dxg-pkg-config-wrapper-0.29.2/bin:/run/wrappers/bin:/run/current-system/sw/bin:$PATH"
$ export PKG_CONFIG_PATH="/nix/store/1l5jgzy26hkjz1y3apn1051asvn42sfn-openssl-3.6.1-dev/lib/pkgconfig"
$ export SNIX_BUILD_SANDBOX_SHELL="/nix/store/d7fc5i7y71rj8cr5jwmaxwjnyvfiybdp-busybox-static-x86_64-unknown-linux-musl-1.37.0/bin/busybox"
$ CARGO_TARGET_DIR=/tmp/mantle-drain-target cargo test -p mantle --bin mantle portable_receipt::tests -- --nocapture
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
   Compiling mantle v0.1.0 (/home/brittonr/git/mantle)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 20.42s
     Running unittests src/main.rs (/tmp/mantle-drain-target/debug/deps/mantle-a789c34159ead64f)

running 31 tests
test portable_receipt::tests::archive_import_without_graph_keeps_why_incomplete ... ok
test portable_receipt::tests::archive_verified_import_persists_graph_evidence_for_why ... ok
test portable_receipt::tests::cli_bundle_export_list_verify_import_covers_human_and_json_paths ... ok
test portable_receipt::tests::cli_bundle_import_conflict_reports_before_persisting_bundle ... ok
test portable_receipt::tests::embedded_sidecar_digest_mismatch_rejects_bundle ... ok
test portable_receipt::tests::same_name_different_key_rejects_trusted_signature_replay ... ok
test portable_receipt::tests::trusted_public_key_verifies_pathinfo_signature_and_snapshot_digest ... ok
test portable_receipt::tests::trusted_signature_verification_rejects_unsigned_output ... ok
...
test result: ok. 31 passed; 0 failed; 0 ignored; 0 measured; 1054 filtered out; finished in 0.03s
```

### receipt-cli-parse-test

```text
$ CARGO_TARGET_DIR=/tmp/mantle-drain-target cargo test -p mantle --bin mantle tests::receipt_bundle_cli_accepts_trusted_public_key_flags -- --nocapture
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.21s
     Running unittests src/main.rs (/tmp/mantle-drain-target/debug/deps/mantle-a789c34159ead64f)

running 1 test
test tests::receipt_bundle_cli_accepts_trusted_public_key_flags ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 1084 filtered out; finished in 0.00s
```

### semantic-graph-focused-tests

```text
$ CARGO_TARGET_DIR=/tmp/mantle-drain-target cargo test -p mantle --bin mantle semantic_graph::tests -- --nocapture
warning: /home/brittonr/git/mantle/Cargo.toml: file `/home/brittonr/git/mantle/src/main.rs` found to be present in multiple build targets:
  * `bin` target `crunch`
  * `bin` target `mantle`
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.22s
     Running unittests src/main.rs (/tmp/mantle-drain-target/debug/deps/mantle-a789c34159ead64f)

running 4 tests
test semantic_graph::tests::names_are_metadata_over_stable_identities ... ok
test semantic_graph::tests::why_links_output_to_recipe_sources_provider_sandbox_and_proofs ... ok
test semantic_graph::tests::incomplete_graph_does_not_invent_edges ... ok
test semantic_graph::tests::dependents_are_reported_by_identity ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 1081 filtered out; finished in 0.00s
```

### formatting-and-diff

```text
$ CARGO_TARGET_DIR=/tmp/mantle-drain-target cargo fmt --check -p mantle -- src/main.rs src/portable_receipt.rs
status: ok (no output)

$ git diff --check
status: ok (no output)
```
