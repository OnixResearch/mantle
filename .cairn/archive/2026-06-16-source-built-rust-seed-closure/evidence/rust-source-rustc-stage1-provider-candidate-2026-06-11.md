# Rust source rustc-stage1 provider candidate boundary

Task-ID: rust-source-rustc-stage1-provider-candidate-2026-06-11
Covers: rust_package_planning.source_built_rust_seed_closure

## Implementation

Mantle now assembles a scratch-only second-stage provider candidate from the synthetic `rustc-stage1` build outputs:

- Copies `rustc-stage1-output/` into `rustc-stage1-provider-candidate/`.
- Writes `share/mantle-rust-provider/receipts/rustc-stage1-build.json` for the stage1 candidate outputs.
- Writes `share/mantle-rust-provider/provider.json` with source identity for `rust-1.91.1` plus the bootstrap provider candidate metadata digest.
- Validates the candidate through the existing provider metadata/artifact/receipt validators.
- Runs the candidate smoke rail and records `rustc-stage1-provider-candidate-smoke/smoke.json` plus smoke stdout/stderr/source/output sidecars.
- Records `rustc-stage1-provider-candidate.json` with `candidate_only: true` and `final_output_written: false`.

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
test result: ok. 44 passed; 0 failed; 0 ignored; 0 measured; 719 filtered out; finished in 15.33s
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
test rust_source_provider::tests::rustc_stage1_provider_candidate_rejects_tampered_artifact ... ok
test rust_source_provider::tests::materializer_prepares_first_stage_boundary_then_fails_closed ... ok

test result: ok. 45 passed; 0 failed; 0 ignored; 0 measured; 719 filtered out; finished in 0.95s
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

test result: ok. 42 passed; 0 failed; 0 ignored; 0 measured; 722 filtered out; finished in 0.09s
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

The second-stage candidate is scratch-local evidence only. Mantle still has no durable final Rust provider output, no provider-backed self-build, and no fixed-point proof. The next slice should advance the remaining Rust source stages or assemble the final provider only when real receipt-bound outputs are available.
