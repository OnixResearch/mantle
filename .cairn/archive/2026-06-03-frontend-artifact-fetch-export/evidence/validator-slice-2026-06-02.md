# Admitted artifact export validator slice — 2026-06-02

## Scope

This evidence covers the first implementation slice for `frontend-artifact-fetch-export`:

- pure request/result/receipt model for admitted frontend artifact export;
- fail-closed validation for missing proof, unsupported ref scheme, artifact/proof mismatch, digest mismatch, unavailable content, and hidden fallback markers;
- positive and negative unit tests for the pure model;
- no CLI/API export seam yet.

## Implementation evidence

Primary implementation paths:

- `src/frontend_artifact_export.rs`
- `src/main.rs` module registration for binary tests

The implementation is functional-core only: it consumes in-memory request, attestation, and content metadata and returns an export report. It performs no filesystem, process, network, or storage I/O.

The receipt hash uses BLAKE3 over a deterministic field preimage and records `receipt_hash_algorithm = "blake3"`.

## Baseline note

Initial direct host Cargo and devshell attempts failed before test execution because the environment lacked the compile-time `SNIX_BUILD_SANDBOX_SHELL` value or used stale target artifacts. The corrected repo-compatible command sets `SNIX_BUILD_SANDBOX_SHELL=/bin/sh`, matching the flake check environment.

## Focused tests

Command:

```text
/home/brittonr/git/mantle $ SNIX_BUILD_SANDBOX_SHELL=/bin/sh CARGO_TARGET_DIR=target/onix-agent-export nix develop -c cargo test --bin mantle frontend_artifact_export
```

Result:

```text
running 9 tests
test frontend_artifact_export::tests::digest_mismatch_fails_closed ... ok
test frontend_artifact_export::tests::hidden_fallback_marker_fails_closed ... ok
test frontend_artifact_export::tests::missing_attestation_fails_before_export ... ok
test frontend_artifact_export::tests::missing_content_fails_closed ... ok
test frontend_artifact_export::tests::export_receipt_serializes_receipt_hash ... ok
test frontend_artifact_export::tests::unsupported_ref_scheme_fails_closed ... ok
test frontend_artifact_export::tests::onix_like_kind_remains_opaque_when_attested ... ok
test frontend_artifact_export::tests::wrong_artifact_ref_fails_closed ... ok
test frontend_artifact_export::tests::valid_admitted_artifact_exports_with_receipt ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 667 filtered out; finished in 0.00s
```

Regression command:

```text
/home/brittonr/git/mantle $ SNIX_BUILD_SANDBOX_SHELL=/bin/sh CARGO_TARGET_DIR=target/onix-agent-export nix develop -c cargo test --bin mantle frontend_artifact_spec
```

Result:

```text
running 10 tests
test frontend_artifact_spec::tests::missing_spec_fails_closed ... ok
test frontend_artifact_spec::tests::manifest_spec_binding_mismatch_fails_closed ... ok
test frontend_artifact_spec::tests::artifact_kind_not_allowed_fails_closed ... ok
test frontend_artifact_spec::tests::hidden_fallback_marker_fails_closed ... ok
test frontend_artifact_spec::tests::spec_hash_mismatch_fails_closed ... ok
test frontend_artifact_spec::tests::onix_like_kind_remains_frontend_data_under_spec ... ok
test frontend_artifact_spec::tests::frontend_semantic_terms_are_data_under_the_declared_spec ... ok
test frontend_artifact_spec::tests::admission_sidecar_serializes_attestation ... ok
test frontend_artifact_spec::tests::valid_frontend_artifact_spec_admits_manifest ... ok
test frontend_artifact_spec::tests::unsupported_validator_kind_fails_closed ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 666 filtered out; finished in 0.00s
```

## Non-claims

This slice does not claim that Mantle can fetch bytes from storage or export artifacts through a CLI. The CLI/API seam and real storage-backed export remain unchecked tasks.
