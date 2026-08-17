# Rust source first-stage env scrub

Task-ID: rust-source-first-stage-env-scrub-2026-06-12
Covers: rust_package_planning.source_built_rust_seed_closure

## Implementation

Mantle now writes a first-stage shell scrub boundary before the mrustc/minicargo/make commands run. The generated `run-mrustc-first-stage.sh` prints `scrubbing inherited Cargo/build-script environment` and unsets a bounded list of inherited Cargo/build-script variables that can otherwise duplicate minicargo-owned crate metadata in `execve` environments.

The bounded unset list covers:

- Cargo package identity and manifest variables: `CARGO_PKG_*`, `CARGO_MANIFEST_DIR`, `CARGO_MANIFEST_PATH`, `CARGO_CRATE_NAME`, `CARGO_BIN_NAME`, and `CARGO_PRIMARY_PACKAGE`;
- Cargo cfg/runtime variables observed in build-script env: `CARGO_CFG_*`, `CARGO_ENCODED_RUSTFLAGS`, `CARGO`, `RUSTFLAGS`, `RUSTC`, and `RUSTDOC`;
- generic build-script variables that minicargo should own per crate: `OUT_DIR`, `TARGET`, `HOST`, `PROFILE`, `DEBUG`, `OPT_LEVEL`, `NUM_JOBS`, and `MRUSTC_LIBDIR`.

The script regression keeps the scrub before the first `minicargo.mk` make invocation and asserts that stage-owned variables such as `RUSTC_TARGET` are not scrubbed.

This still does not produce a real Rust provider. It only removes the current `rustc_apfloat` env-leak candidate before rerunning the real route.

## Validation

```text
$ cargo test -p mantle --bin mantle rust_source_provider -- --nocapture --test-threads=1
test result: ok. 55 passed; 0 failed; 0 ignored; 0 measured; 719 filtered out; finished in 7.31s
$ rustfmt --check src/rust_source_provider.rs
rustfmt ok
$ git diff --check
git diff --check ok
```
