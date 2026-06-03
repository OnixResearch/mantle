# Executor/report slice evidence (2026-06-02)

## Implemented scope

- Added a generic frontend-neutral validator executor for `manifest-kind-allowlist-v1` specs.
  - The spec material is content-hashed with BLAKE3 before execution.
  - The executor parses only frontend-provided allowlist JSON and validates `manifest.kind` as data.
  - It performs no ambient host process execution and contains no Onix/NixOS role/tag/provider semantics.
- Added sidecar/receipt JSON rendering through `render_frontend_artifact_admission_sidecar`.
- Added populated build-report rendering through `render_build_json_report_with_frontend_artifact_attestations` while keeping the default build report empty when no attestation is supplied.
- Added boundary tests that Onix-like, NixOS-like, role, tag, and provider strings are accepted only as data under the declared spec allowlist.

## Focused validator/executor tests

Command:

```sh
PATH=$HOME/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin:/nix/store/97vplpbajnr7x03fqh9biz5v6960sv22-clang-wrapper-21.1.8/bin:/nix/store/27ddg2m33wnp6xrr7il86yjyr41n88ya-mold-2.40.4/bin:/nix/store/rvp7qlpf5jqvdckjy1afjb6aha6j8dxg-pkg-config-wrapper-0.29.2/bin:$PATH \
  CARGO_TARGET_DIR=target/pi-frontend-artifact-test \
  PKG_CONFIG_PATH=/nix/store/1l5jgzy26hkjz1y3apn1051asvn42sfn-openssl-3.6.1-dev/lib/pkgconfig \
  SNIX_BUILD_SANDBOX_SHELL=/nix/store/4mdqc2snfiihr6r61ln1rqs4fis6br9b-busybox-static-x86_64-unknown-linux-musl-1.36.1/bin/busybox \
  cargo test -p mantle --bin mantle frontend_artifact_spec -- --nocapture
```

Output excerpt:

```text
running 10 tests
test frontend_artifact_spec::tests::unsupported_validator_kind_fails_closed ... ok
test frontend_artifact_spec::tests::missing_spec_fails_closed ... ok
test frontend_artifact_spec::tests::manifest_spec_binding_mismatch_fails_closed ... ok
test frontend_artifact_spec::tests::hidden_fallback_marker_fails_closed ... ok
test frontend_artifact_spec::tests::spec_hash_mismatch_fails_closed ... ok
test frontend_artifact_spec::tests::artifact_kind_not_allowed_fails_closed ... ok
test frontend_artifact_spec::tests::onix_like_kind_remains_frontend_data_under_spec ... ok
test frontend_artifact_spec::tests::frontend_semantic_terms_are_data_under_the_declared_spec ... ok
test frontend_artifact_spec::tests::valid_frontend_artifact_spec_admits_manifest ... ok
test frontend_artifact_spec::tests::admission_sidecar_serializes_attestation ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 657 filtered out; finished in 0.00s
```

## Populated report attestation test

Command:

```sh
PATH=$HOME/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin:/nix/store/97vplpbajnr7x03fqh9biz5v6960sv22-clang-wrapper-21.1.8/bin:/nix/store/27ddg2m33wnp6xrr7il86yjyr41n88ya-mold-2.40.4/bin:/nix/store/rvp7qlpf5jqvdckjy1afjb6aha6j8dxg-pkg-config-wrapper-0.29.2/bin:$PATH \
  CARGO_TARGET_DIR=target/pi-frontend-artifact-test \
  PKG_CONFIG_PATH=/nix/store/1l5jgzy26hkjz1y3apn1051asvn42sfn-openssl-3.6.1-dev/lib/pkgconfig \
  SNIX_BUILD_SANDBOX_SHELL=/nix/store/4mdqc2snfiihr6r61ln1rqs4fis6br9b-busybox-static-x86_64-unknown-linux-musl-1.36.1/bin/busybox \
  cargo test -p mantle --bin mantle render_build_json_report_serializes_frontend_artifact_attestation -- --nocapture
```

Output excerpt:

```text
running 1 test
test build_report::tests::render_build_json_report_serializes_frontend_artifact_attestation ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 666 filtered out; finished in 0.00s
```

## Regression report tests

Commands:

```sh
cargo test -p mantle --bin mantle render_build_json_report_serializes_full_substitution_fields_stably -- --nocapture
cargo test -p mantle --bin mantle build_json_report_includes_artifact_attestation_reference -- --nocapture
```

Output excerpts:

```text
test build_report::tests::render_build_json_report_serializes_full_substitution_fields_stably ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 666 filtered out; finished in 0.00s
```

```text
test build_report::tests::build_json_report_includes_artifact_attestation_reference ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 666 filtered out; finished in 0.00s
```
