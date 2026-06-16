# Rust source first-stage provider candidate smoke

Task-ID: rust-source-first-stage-provider-smoke-2026-06-11
Covers: rust_package_planning.source_built_rust_seed_closure

## Implementation

Mantle now smoke-tests the scratch-local first-stage provider candidate before preserving the fail-closed blocker:

- The materializer validates the first-stage provider candidate metadata and receipts.
- It runs `smoke_rust_source_provider(...)` using only the candidate provider paths and a hermetic smoke invocation.
- It persists candidate smoke evidence under `mrustc-first-stage-provider-candidate-smoke/` in scratch, including stdout, stderr, source, output, copied metadata, and `smoke.json`.
- The requested final `--output-dir` is still not written, and the command still returns the existing fail-closed blocker.

This is candidate evidence only. It does not replace the real provider smoke task because the candidate is not promoted to a durable final provider output and no fixed-point proof has consumed it.

## Baseline before changes

Commands:

```sh
PATH=/home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin:/nix/store/97vplpbajnr7x03fqh9biz5v6960sv22-clang-wrapper-21.1.8/bin:/nix/store/rvp7qlpf5jqvdckjy1afjb6aha6j8dxg-pkg-config-wrapper-0.29.2/bin:/run/current-system/sw/bin:/usr/bin:/bin \
PKG_CONFIG_PATH=/nix/store/1l5jgzy26hkjz1y3apn1051asvn42sfn-openssl-3.6.1-dev/lib/pkgconfig \
SNIX_BUILD_SANDBOX_SHELL=/bin/sh \
RUSTFLAGS='-C link-arg=-Wl,--allow-multiple-definition' \
cargo test -p mantle --bin mantle rust_source_provider -- --nocapture

PATH=/home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin:/nix/store/97vplpbajnr7x03fqh9biz5v6960sv22-clang-wrapper-21.1.8/bin:/nix/store/rvp7qlpf5jqvdckjy1afjb6aha6j8dxg-pkg-config-wrapper-0.29.2/bin:/run/current-system/sw/bin:/usr/bin:/bin \
PKG_CONFIG_PATH=/nix/store/1l5jgzy26hkjz1y3apn1051asvn42sfn-openssl-3.6.1-dev/lib/pkgconfig \
SNIX_BUILD_SANDBOX_SHELL=/bin/sh \
RUSTFLAGS='-C link-arg=-Wl,--allow-multiple-definition' \
cargo test -p mantle --bin mantle source_toolchain_closure -- --nocapture
```

Output summaries:

```text
test result: ok. 43 passed; 0 failed; 0 ignored; 0 measured; 719 filtered out; finished in 0.34s
test result: ok. 42 passed; 0 failed; 0 ignored; 0 measured; 720 filtered out; finished in 0.08s
```

## Validation after changes

### provider candidate smoke tests

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
test rust_source_provider::tests::first_stage_provider_candidate_rejects_tampered_artifact ... ok
test rust_source_provider::tests::materializer_prepares_first_stage_boundary_then_fails_closed ... ok
test result: ok. 43 passed; 0 failed; 0 ignored; 0 measured; 719 filtered out; finished in 1.26s
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
test result: ok. 42 passed; 0 failed; 0 ignored; 0 measured; 720 filtered out; finished in 0.24s
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

The provider candidate smoke proves the candidate can satisfy the smoke rail in synthetic coverage, but the candidate remains scratch-local. The final provider output, provider-backed self-build, and fixed-point proof remain blocked; `not-source-built-toolchain-closure` remains.
