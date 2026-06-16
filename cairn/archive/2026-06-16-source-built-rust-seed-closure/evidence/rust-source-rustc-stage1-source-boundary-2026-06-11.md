# Rust source rustc-stage1 source boundary

Task-ID: rust-source-rustc-stage1-source-boundary-2026-06-11
Covers: rust_package_planning.source_built_rust_seed_closure

## Implementation

Mantle now advances past the mrustc first-stage candidate far enough to bind the first `rustc-stage1` source boundary:

- Selects the first `rustc-stage1` route stage whose `bootstrap_stage_id` is the validated mrustc first-stage candidate.
- Writes `rustc-stage1-plan.json` with route digests, bootstrap provider candidate digest/policy, stage ID, source IDs, expected roles, and scratch paths.
- Fetches/verifies/extracts the `rust-1.91.1` source tarball into `rustc-stage1-sources/` and records `rustc-stage1-sources.json`.
- Adds a negative digest-mismatch test proving a tampered `rust-1.91.1` declaration fails closed after candidate assembly but before any final provider output.

This still does not build the `rustc-stage1` compiler. No final provider output is written and `not-source-built-toolchain-closure` remains.

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
test result: ok. 44 passed; 0 failed; 0 ignored; 0 measured; 719 filtered out; finished in 13.48s
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
test result: ok. 42 passed; 0 failed; 0 ignored; 0 measured; 721 filtered out; finished in 0.40s
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

The next stage source is verified and extracted, but the `rustc-stage1` build step has not been implemented. The provider remains scratch-local/candidate-only and no fixed-point proof has consumed it.
