# Rust source first-stage mrustc/minicargo build boundary

Task-ID: rust-source-first-stage-mrustc-build-2026-06-11
Covers: rust_package_planning.source_built_rust_seed_closure

## Implementation

Mantle now runs the verified first-stage source boundary far enough to attempt the mrustc/minicargo build:

- `run-mrustc-first-stage.sh` checks the verified source manifest, extracted `mrustc-*` and `rust-*` source trees, and the verified archives.
- The script copies the verified Rust source archive into the extracted mrustc tree as `rustc-<version>-src.tar.gz`, then invokes `make CXXFLAGS=...` and `make -f minicargo.mk bin/minicargo`.
- The materializer captures stdout/stderr in `mrustc-first-stage-build.log` and fails closed with a log tail when the build script fails.
- On build-script success, the materializer validates `bin/mrustc` and `bin/minicargo`, records BLAKE3 digests in `mrustc-first-stage-build.json`, and then still returns the existing provider blocker because Rust compiler/sysroot output metadata and receipts are not implemented yet.
- A Nix-store gnumake fallback is used only as a host convenience for this non-claiming build attempt when `make` is absent from `PATH`; it is not provider evidence and does not remove the source-built Rust non-claim.

This still does not build `rustc`, `cargo`, host rustlib, target rustlib, final provider metadata, or provider receipts. The real provider task remains unchecked.

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
test result: ok. 42 passed; 0 failed; 0 ignored; 0 measured; 719 filtered out; finished in 0.05s
test result: ok. 42 passed; 0 failed; 0 ignored; 0 measured; 719 filtered out; finished in 0.04s
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
test rust_source_provider::tests::materializer_prepares_first_stage_boundary_then_fails_closed ... ok
test rust_source_provider::tests::materializer_rejects_first_stage_source_digest_mismatch_without_output ... ok
test rust_source_provider::tests::first_stage_script_fails_before_provider_output_when_sources_missing ... ok
test result: ok. 42 passed; 0 failed; 0 ignored; 0 measured; 719 filtered out; finished in 0.50s
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
test result: ok. 42 passed; 0 failed; 0 ignored; 0 measured; 719 filtered out; finished in 0.10s
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

The first-stage script can now run mrustc/minicargo make targets and record product digests, but the materializer still stops before the Rust 1.90 compiler/sysroot build and before final provider metadata/receipt emission. `not-source-built-toolchain-closure` remains.
