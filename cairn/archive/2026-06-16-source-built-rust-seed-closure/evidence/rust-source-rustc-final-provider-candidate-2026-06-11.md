# Rust source final provider candidate boundary

Task-ID: rust-source-rustc-final-provider-candidate-2026-06-11
Covers: rust_package_planning.source_built_rust_seed_closure

## Implementation

Mantle now advances the bounded Rust source route through a scratch-only `rust-1.94.0-final` provider candidate:

- Runs the existing bounded stage1 chain through `rust-1.93.1-stage1`.
- Selects the `rustc-final` route stage whose bootstrap provider is the final stage1 candidate.
- Verifies the `rust-1.94.0` source archive before writing final-stage source manifests.
- Runs a final-stage build boundary that produces `bin/rustc`, `bin/cargo`, `bin/rustdoc`, host rustlib, and target rustlib under scratch.
- Assembles a scratch-local final provider candidate with validated metadata and `share/mantle-rust-provider/receipts/build.json`.
- Runs candidate-only smoke evidence against the final candidate.
- Adds negative coverage for bad `rust-1.94.0` source digests and tampered final `rustdoc` artifacts.

This remains candidate-only. It does not write the requested final `--output-dir`, does not launch a provider-backed self-build, does not claim a fixed point, and does not remove `not-source-built-toolchain-closure`.

## Validation after changes

### rust source provider tests

Command:

```sh
PATH=/home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin:/nix/store/97vplpbajnr7x03fqh9biz5v6960sv22-clang-wrapper-21.1.8/bin:/nix/store/1sw8whfl5gfblp6r9qdkiw1b4j9fgwar-mold-2.40.4/bin:/nix/store/rvp7qlpf5jqvdckjy1afjb6aha6j8dxg-pkg-config-wrapper-0.29.2/bin:$PATH \
PKG_CONFIG_PATH=/nix/store/1l5jgzy26hkjz1y3apn1051asvn42sfn-openssl-3.6.1-dev/lib/pkgconfig \
SNIX_BUILD_SANDBOX_SHELL=/nix/store/d7fc5i7y71rj8cr5jwmaxwjnyvfiybdp-busybox-static-x86_64-unknown-linux-musl-1.37.0/bin/busybox \
cargo test -p mantle --bin mantle rust_source_provider -- --nocapture
```

Output summary:

```text
test rust_source_provider::tests::materializer_rejects_rustc_final_source_digest_mismatch_without_output ... ok
test rust_source_provider::tests::rustc_final_provider_candidate_rejects_tampered_artifact ... ok
test rust_source_provider::tests::materializer_prepares_first_stage_boundary_then_fails_closed ... ok

test result: ok. 51 passed; 0 failed; 0 ignored; 0 measured; 719 filtered out; finished in 1.81s
```

### route/core regression tests

Command:

```sh
PATH=/home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin:/nix/store/97vplpbajnr7x03fqh9biz5v6960sv22-clang-wrapper-21.1.8/bin:/nix/store/1sw8whfl5gfblp6r9qdkiw1b4j9fgwar-mold-2.40.4/bin:/nix/store/rvp7qlpf5jqvdckjy1afjb6aha6j8dxg-pkg-config-wrapper-0.29.2/bin:$PATH \
PKG_CONFIG_PATH=/nix/store/1l5jgzy26hkjz1y3apn1051asvn42sfn-openssl-3.6.1-dev/lib/pkgconfig \
SNIX_BUILD_SANDBOX_SHELL=/nix/store/d7fc5i7y71rj8cr5jwmaxwjnyvfiybdp-busybox-static-x86_64-unknown-linux-musl-1.37.0/bin/busybox \
cargo test -p mantle --bin mantle source_toolchain_closure -- --nocapture
```

Output summary:

```text
test source_toolchain_closure::tests::checked_in_rust_source_plan_loads_stagex_mrustc_route ... ok

test result: ok. 42 passed; 0 failed; 0 ignored; 0 measured; 728 filtered out; finished in 0.27s
```

### formatting

Command:

```sh
PATH=/home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin:$PATH rustfmt --check src/rust_source_provider.rs
```

Output summary:

```text
completed successfully
```

### whitespace diff check

Command:

```sh
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
  "issues": [],
  "layout": "cairn",
  "policy": "cairn-default",
  "stage": "tasks",
  "valid": true,
  "verdict": "PASS"
}
```

## Remaining blocker

The `rust-1.94.0-final` provider candidate is still scratch-local evidence only. Mantle still has no durable final Rust provider output, no provider-backed self-build, and no fixed-point proof. `not-source-built-toolchain-closure` remains.
