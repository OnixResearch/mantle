# Rust source chained rustc-stage1 provider candidate boundary

Task-ID: rust-source-rustc-stage1-chain-2026-06-11
Covers: rust_package_planning.source_built_rust_seed_closure

## Implementation

Mantle now advances one more bounded Rust source stage after the first `rustc-stage1` candidate:

- Refactors the `rustc-stage1` runner to consume a generic previous provider candidate.
- Preserves the existing top-level `rust-1.91.1-stage1` scratch paths for compatibility with existing evidence.
- Adds stage-scoped scratch paths under `rustc-stage1-chain/rust-1.92.0-stage1/` for the chained stage.
- Runs `rust-1.92.0-stage1` with the previous `rust-1.91.1-stage1` provider candidate as `MANTLE_BOOTSTRAP_PROVIDER`.
- Validates the chained stage output `bin/rustc`, `bin/cargo`, host rustlib, and target rustlib.
- Assembles a scratch-local chained provider candidate with metadata and `share/mantle-rust-provider/receipts/rustc-stage1-build.json`.
- Validates the chained candidate metadata/artifacts/receipt payload and runs the candidate smoke rail.
- Adds negative coverage for chained source digest mismatch and chained candidate artifact tampering.

This remains candidate-only. It does not write the requested final `--output-dir`, does not launch a provider-backed self-build, and does not claim a fixed point or remove `not-source-built-toolchain-closure`.

## Baseline before changes

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
test result: ok. 45 passed; 0 failed; 0 ignored; 0 measured; 719 filtered out; finished in 0.37s
```

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
test rust_source_provider::tests::materializer_rejects_chained_rustc_stage1_source_digest_mismatch_without_output ... ok
test rust_source_provider::tests::chained_rustc_stage1_provider_candidate_rejects_tampered_artifact ... ok
test rust_source_provider::tests::materializer_prepares_first_stage_boundary_then_fails_closed ... ok

test result: ok. 47 passed; 0 failed; 0 ignored; 0 measured; 719 filtered out; finished in 1.68s
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

test result: ok. 42 passed; 0 failed; 0 ignored; 0 measured; 724 filtered out; finished in 0.04s
```

### formatting/checks

Command:

```sh
PATH=/home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin:/run/current-system/sw/bin:/usr/bin:/bin rustfmt --check src/rust_source_provider.rs && git diff --check
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
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```

## Remaining blocker

The chained `rust-1.92.0-stage1` candidate is still scratch-local evidence only. Mantle still has no durable final Rust provider output, no provider-backed self-build, and no fixed-point proof. The next slice should continue the route to the next source stage (`rust-1.93.1-stage1`) or begin final-provider assembly only after the remaining receipt-bound source stages exist.
