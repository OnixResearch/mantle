# Rust source rustc-stage1 synthetic build boundary

Task-ID: rust-source-rustc-stage1-build-2026-06-11
Covers: rust_package_planning.source_built_rust_seed_closure

## Implementation

Mantle now runs a bounded `rustc-stage1` build boundary after verifying `rust-1.91.1` sources:

- Writes an executable `run-rustc-stage1.sh` that consumes the scratch-local mrustc provider candidate as `MANTLE_BOOTSTRAP_PROVIDER`.
- Validates the bootstrap provider candidate has `bin/rustc`, `bin/cargo`, and target rustlib before launching the stage script.
- Runs a source-provided stage build helper in synthetic coverage and captures `rustc-stage1-build.log`.
- Validates stage1 `bin/rustc`, `bin/cargo`, host rustlib, and target rustlib outputs.
- Records output BLAKE3 digests in `rustc-stage1-build.json`.

This remains a build-boundary proof only. It does not assemble a second-stage provider candidate, write final `--output-dir`, run provider-backed self-build, or remove `not-source-built-toolchain-closure`.

## Validation after changes

### rust source provider tests

Command:

```sh
PATH=/home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin:/nix/store/97vplpbajnr7x03fqh9biz5v6960sv22-clang-wrapper-21.1.8/bin:/nix/store/rvp7qlpf5jqvdckjy1afjb6aha6j8dxg-pkg-config-wrapper-0.29.2/bin:/run/current-system/sw/bin:/usr/bin:/bin \
PKG_CONFIG_PATH=/nix/store/1l5jgzy26hkjz1y3apn1051asvn42sfn-openssl-3.6.1-dev/lib/pkgconfig \
SNIX_BUILD_SANDBOX_SHELL=/bin/sh \
RUSTFLAGS='-C link-arg=-Wl,--allow-multiple-definition' \
cargo test -p mantle --bin mantle rust_source_provider -- --nocapture
```

Output summary:

```text
test rust_source_provider::tests::materializer_rejects_rustc_stage1_source_digest_mismatch_without_output ... ok
test rust_source_provider::tests::materializer_prepares_first_stage_boundary_then_fails_closed ... ok
test result: ok. 44 passed; 0 failed; 0 ignored; 0 measured; 719 filtered out; finished in 8.38s
```

### route/core regression tests

Command:

```sh
PATH=/home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin:/nix/store/97vplpbajnr7x03fqh9biz5v6960sv22-clang-wrapper-21.1.8/bin:/nix/store/rvp7qlpf5jqvdckjy1afjb6aha6j8dxg-pkg-config-wrapper-0.29.2/bin:/run/current-system/sw/bin:/usr/bin:/bin \
PKG_CONFIG_PATH=/nix/store/1l5jgzy26hkjz1y3apn1051asvn42sfn-openssl-3.6.1-dev/lib/pkgconfig \
SNIX_BUILD_SANDBOX_SHELL=/bin/sh \
RUSTFLAGS='-C link-arg=-Wl,--allow-multiple-definition' \
cargo test -p mantle --bin mantle source_toolchain_closure -- --nocapture
```

Output summary:

```text
test source_toolchain_closure::tests::checked_in_rust_source_plan_loads_stagex_mrustc_route ... ok
test result: ok. 42 passed; 0 failed; 0 ignored; 0 measured; 721 filtered out; finished in 0.29s
```

### formatting/checks

Command:

```sh
export PATH=/home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin:/run/current-system/sw/bin:/usr/bin:/bin
rustfmt src/rust_source_provider.rs
rustfmt --check src/rust_source_provider.rs
git diff --check
```

Output summary:

```text
completed successfully
```

## Lifecycle validation

### cairn validate

Command:

```sh
nix run path:/home/brittonr/git/cairn#cairn -- validate --root .
```

Output:

```json
{
  "change_issues": [],
  "changes": 1,
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "spec_issues": [],
  "specs_validated": 6,
  "valid": true
}
```

### cairn tasks gate

Command:

```sh
nix run path:/home/brittonr/git/cairn#cairn -- gate tasks source-built-rust-seed-closure --root .
```

Output summary:

```json
{
  "change": "source-built-rust-seed-closure",
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```

## Remaining blocker

The `rustc-stage1` build boundary now produces and validates synthetic stage1 outputs, but Mantle still has no durable final Rust provider output and no provider-backed fixed-point proof. The next slice should assemble a second-stage candidate from these outputs or advance the route to the next Rust source stage while preserving fail-closed behavior.
