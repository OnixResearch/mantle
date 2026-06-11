# Rust source final chained rustc-stage1 provider candidate boundary

Task-ID: rust-source-rustc-stage1-final-chain-2026-06-11
Covers: rust_package_planning.source_built_rust_seed_closure

## Implementation

Mantle now advances the bounded Rust source stage1 chain through `rust-1.93.1-stage1`:

- Refactors the `rustc-stage1` runner into a bounded chain loop with a fixed maximum stage count.
- Keeps `rust-1.91.1-stage1` at the established top-level scratch paths.
- Keeps chained stages under `rustc-stage1-chain/<stage-id>/`.
- Runs `rust-1.92.0-stage1` using the `rust-1.91.1-stage1` candidate as bootstrap provider.
- Runs `rust-1.93.1-stage1` using the `rust-1.92.0-stage1` candidate as bootstrap provider.
- Validates each chained stage output `bin/rustc`, `bin/cargo`, host rustlib, and target rustlib.
- Assembles scratch-local provider candidates with metadata and `share/mantle-rust-provider/receipts/rustc-stage1-build.json`.
- Validates metadata/artifacts/receipt payloads and runs candidate smoke rails.
- Adds negative coverage for `rust-1.93.1` source digest mismatch and final chained candidate artifact tampering.

This remains candidate-only. It does not write the requested final `--output-dir`, does not run the `rust-1.94.0-final` provider assembly, does not launch a provider-backed self-build, and does not claim a fixed point or remove `not-source-built-toolchain-closure`.

## Baseline before changes

The pre-edit baseline was run against parent commit `51bd67d5` in a temporary worktree because the working tree was already edited when baseline evidence was collected.

Command:

```sh
git worktree add --detach /tmp/mantle-rust-source-baseline-51bd67d5 51bd67d5
cd /tmp/mantle-rust-source-baseline-51bd67d5
PATH=/home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin:/nix/store/97vplpbajnr7x03fqh9biz5v6960sv22-clang-wrapper-21.1.8/bin:/nix/store/rvp7qlpf5jqvdckjy1afjb6aha6j8dxg-pkg-config-wrapper-0.29.2/bin:/run/current-system/sw/bin:/usr/bin:/bin \
PKG_CONFIG_PATH=/nix/store/1l5jgzy26hkjz1y3apn1051asvn42sfn-openssl-3.6.1-dev/lib/pkgconfig \
SNIX_BUILD_SANDBOX_SHELL=/bin/sh \
RUSTFLAGS='-C link-arg=-Wl,--allow-multiple-definition' \
cargo test -p mantle --bin mantle rust_source_provider -- --nocapture
```

Output summary:

```text
test result: ok. 47 passed; 0 failed; 0 ignored; 0 measured; 719 filtered out; finished in 0.52s
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
test rust_source_provider::tests::materializer_rejects_final_chained_rustc_stage1_source_digest_mismatch_without_output ... ok
test rust_source_provider::tests::final_chained_rustc_stage1_provider_candidate_rejects_tampered_artifact ... ok
test rust_source_provider::tests::materializer_prepares_first_stage_boundary_then_fails_closed ... ok

test result: ok. 49 passed; 0 failed; 0 ignored; 0 measured; 719 filtered out; finished in 1.07s
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

test result: ok. 42 passed; 0 failed; 0 ignored; 0 measured; 726 filtered out; finished in 0.10s
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

The `rust-1.93.1-stage1` candidate is still scratch-local evidence only. Mantle still has no durable final Rust provider output, no `rust-1.94.0-final` provider assembly, no provider-backed self-build, and no fixed-point proof. The next bounded slice should assemble a final provider candidate from `rust-1.94.0-final` outputs while still keeping final promotion blocked until provider-backed self-build and fixed-point proof exist.
