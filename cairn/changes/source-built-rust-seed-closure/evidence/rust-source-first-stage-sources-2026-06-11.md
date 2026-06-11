# Rust source first-stage source acquisition boundary

Task-ID: rust-source-first-stage-sources-2026-06-11
Covers: rust_package_planning.source_built_rust_seed_closure

## Implementation

Mantle now advances the executable mrustc first-stage boundary from plan-only scaffolding to verified source acquisition:

- `src/rust_source_provider.rs` fetches first-stage source tarballs from `file://`, `http://`, or `https://` URLs with bounded HTTP retry/size limits.
- It verifies each declared `sha256_hex` before writing the archive under the scratch `archives/` directory.
- It extracts tar.gz/tgz archives through Rust `flate2` + `tar`, strips the top-level archive component, rejects unsafe archive paths, and writes extracted trees under scratch `sources/<source-id>/`.
- It writes `mrustc-first-stage-sources.json` with source URLs, archive paths, raw SHA-256 digests, extracted paths, extracted BLAKE3 digests, and archive entry counts.
- The generated `run-mrustc-first-stage.sh` now checks the verified source manifest and extracted source directories before failing closed at the unimplemented build step.

This still does not build `mrustc`, `rustc`, `cargo`, rustlib, provider metadata, or provider receipts. The materializer still returns the source-built Rust provider blocker after verified source acquisition succeeds.

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
test result: ok. 41 passed; 0 failed; 0 ignored; 0 measured; 719 filtered out; finished in 10.50s
```

## Validation after changes

### first-stage provider boundary tests

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
test rust_source_provider::tests::materializer_rejects_first_stage_source_digest_mismatch_without_output ... ok
test rust_source_provider::tests::materializer_prepares_first_stage_boundary_then_fails_closed ... ok
test rust_source_provider::tests::first_stage_script_fails_before_provider_output_when_sources_missing ... ok
test result: ok. 42 passed; 0 failed; 0 ignored; 0 measured; 719 filtered out; finished in 0.05s
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
test result: ok. 42 passed; 0 failed; 0 ignored; 0 measured; 719 filtered out; finished in 0.04s
```

### formatting check

Command:

```sh
export PATH=/home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin:/run/current-system/sw/bin:/usr/bin:/bin
rustfmt --check src/rust_source_provider.rs
rustfmt --check --config skip_children=true src/main.rs
```

Output summary:

```text
completed successfully
```

Note: plain `rustfmt --check src/main.rs src/rust_source_provider.rs` still traverses child modules and reports a pre-existing `src/build_report.rs` formatting diff; this slice reverted that unrelated rustfmt churn and used `skip_children=true` for the touched main-shell file.

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

The materializer now proves source acquisition and extraction for the first mrustc stage, but it still fails closed before any compiler build. The unchecked provider materialization, real provider smoke, and fixed-point proof tasks remain blocked until Mantle actually builds mrustc/Rust stages and emits validated provider metadata plus receipts.
