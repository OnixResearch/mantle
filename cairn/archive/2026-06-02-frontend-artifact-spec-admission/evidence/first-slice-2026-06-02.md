# First slice implementation evidence (2026-06-02)

## Implemented scope

- Pure frontend artifact spec reference model and artifact manifest model in `src/frontend_artifact_spec.rs`.
- Pure admission validator for missing spec, empty fields, unsupported validator kind, unsupported/hash-mismatch BLAKE3 spec hash, spec-binding mismatch, and hidden-fallback marker.
- JSON build report shape now includes `frontend_artifact_attestations` as the report-level location for future spec-validation attestations.
- Positive/negative focused tests cover valid admission, missing spec, unsupported validator, hash mismatch, spec-binding mismatch, hidden fallback, and Onix-like kind as frontend data.

## Baseline note

Initial baseline attempt from the ambient Onix shell failed before tests because Mantle's `.cargo/config.toml` requires `clang` and the ambient PATH lacked it:

```text
COMMAND /home/brittonr/git/mantle $ cargo test -p mantle build_json_report_includes_artifact_attestation_reference -- --nocapture
STATUS 101
error: linker `clang` not found
```

A second attempt with the ambient stable toolchain was also invalid for Mantle because the repo requires nightly (`#![feature(register_tool)]`). Verification below uses Mantle's nightly toolchain and explicit dev tool paths.

## Focused positive/negative validator tests

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
running 7 tests
test frontend_artifact_spec::tests::onix_like_kind_remains_frontend_data_under_spec ... ok
test frontend_artifact_spec::tests::missing_spec_fails_closed ... ok
test frontend_artifact_spec::tests::unsupported_validator_kind_fails_closed ... ok
test frontend_artifact_spec::tests::manifest_spec_binding_mismatch_fails_closed ... ok
test frontend_artifact_spec::tests::hidden_fallback_marker_fails_closed ... ok
test frontend_artifact_spec::tests::spec_hash_mismatch_fails_closed ... ok
test frontend_artifact_spec::tests::valid_frontend_artifact_spec_admits_manifest ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 656 filtered out; finished in 0.00s
STATUS 0
```

## Build report shape tests

Command:

```sh
PATH=$HOME/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin:/nix/store/97vplpbajnr7x03fqh9biz5v6960sv22-clang-wrapper-21.1.8/bin:/nix/store/27ddg2m33wnp6xrr7il86yjyr41n88ya-mold-2.40.4/bin:/nix/store/rvp7qlpf5jqvdckjy1afjb6aha6j8dxg-pkg-config-wrapper-0.29.2/bin:$PATH \
  CARGO_TARGET_DIR=target/pi-frontend-artifact-test \
  PKG_CONFIG_PATH=/nix/store/1l5jgzy26hkjz1y3apn1051asvn42sfn-openssl-3.6.1-dev/lib/pkgconfig \
  SNIX_BUILD_SANDBOX_SHELL=/nix/store/4mdqc2snfiihr6r61ln1rqs4fis6br9b-busybox-static-x86_64-unknown-linux-musl-1.36.1/bin/busybox \
  cargo test -p mantle --bin mantle build_json_report_includes_artifact_attestation_reference -- --nocapture
```

Output excerpt:

```text
running 1 test
test build_report::tests::build_json_report_includes_artifact_attestation_reference ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 662 filtered out; finished in 0.00s
STATUS 0
```

Command:

```sh
PATH=$HOME/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin:/nix/store/97vplpbajnr7x03fqh9biz5v6960sv22-clang-wrapper-21.1.8/bin:/nix/store/27ddg2m33wnp6xrr7il86yjyr41n88ya-mold-2.40.4/bin:/nix/store/rvp7qlpf5jqvdckjy1afjb6aha6j8dxg-pkg-config-wrapper-0.29.2/bin:$PATH \
  CARGO_TARGET_DIR=target/pi-frontend-artifact-test \
  PKG_CONFIG_PATH=/nix/store/1l5jgzy26hkjz1y3apn1051asvn42sfn-openssl-3.6.1-dev/lib/pkgconfig \
  SNIX_BUILD_SANDBOX_SHELL=/nix/store/4mdqc2snfiihr6r61ln1rqs4fis6br9b-busybox-static-x86_64-unknown-linux-musl-1.36.1/bin/busybox \
  cargo test -p mantle --bin mantle render_build_json_report_serializes_full_substitution_fields_stably -- --nocapture
```

Output excerpt:

```text
running 1 test
test build_report::tests::render_build_json_report_serializes_full_substitution_fields_stably ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 662 filtered out; finished in 0.00s
STATUS 0
```
