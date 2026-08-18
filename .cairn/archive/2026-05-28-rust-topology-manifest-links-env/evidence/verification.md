# Verification

Task-ID: V1
Covers: rust_package_planning.native_manifest_links_env

## Commands

Baseline before code changes:

```sh
env SNIX_BUILD_SANDBOX_SHELL=/nix/store/8mf4s8c4xjvlkj12p299qylrb30g7zzh-busybox-static-x86_64-unknown-linux-musl-1.37.0/bin/busybox CARGO_TARGET_DIR=/tmp/mantle-selected-host-target CARGO_INCREMENTAL=0 nix develop -c cargo test -p mantle --bin mantle native_cargo_package_env -- --nocapture
```

Post-change focused tests:

```sh
nix develop -c rustfmt src/rust_plan.rs
env SNIX_BUILD_SANDBOX_SHELL=/nix/store/8mf4s8c4xjvlkj12p299qylrb30g7zzh-busybox-static-x86_64-unknown-linux-musl-1.37.0/bin/busybox CARGO_TARGET_DIR=/tmp/mantle-selected-host-target CARGO_INCREMENTAL=0 nix develop -c cargo test -p mantle --bin mantle native_cargo_package_env -- --nocapture
env SNIX_BUILD_SANDBOX_SHELL=/nix/store/8mf4s8c4xjvlkj12p299qylrb30g7zzh-busybox-static-x86_64-unknown-linux-musl-1.37.0/bin/busybox CARGO_TARGET_DIR=/tmp/mantle-selected-host-target CARGO_INCREMENTAL=0 nix develop -c cargo test -p mantle --bin mantle native_host_derivation_carries_manifest_package_name_for_build_script_env -- --nocapture
```

Dirty self-probe:

```sh
pueue task 67: cargo run -q -p mantle --bin mantle -- --json rust-plan --root . --cargo /home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/cargo --rustc /home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/rustc --target x86_64-unknown-linux-gnu --execute-topology --execution-output-root target/mantle-self-rust-plan-probe-manifest-links-dirty2/execution
```

## Results

- Baseline `native_cargo_package_env`: `2 passed`.
- Post-change `native_cargo_package_env`: `2 passed`.
- Post-change `native_host_derivation_carries_manifest_package_name_for_build_script_env`: `1 passed`.
- Dirty self-probe receipt: `target/mantle-self-rust-plan-probe-manifest-links-dirty2/receipt.json`.
- Dirty self-probe status: `topology_execution=blocked`, `topology_unit_executions=201`, `metadata_runs=63`.
- `ring@0.17.14` metadata run: `success`.
- Previous `CARGO_MANIFEST_LINKS` unwrap panic is absent.
- New remaining blocker: `rustc-failed` compiling `crates/crunch-project/src/error.rs`, with `thiserror::Error` derive expansion failing to find `thiserror::__private` and missing `as_dyn_error` on `serde_json::Error`.

Clean self-probe:

```sh
pueue task 68: cargo run -q -p mantle --bin mantle -- --json rust-plan --root . --cargo /home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/cargo --rustc /home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/rustc --target x86_64-unknown-linux-gnu --execute-topology --execution-output-root target/mantle-self-rust-plan-probe-after-27794c0b-clean/execution
```

Clean self-probe result:

- Receipt: `target/mantle-self-rust-plan-probe-after-27794c0b-clean/receipt.json`.
- HEAD: `27794c0b4ff5e7448c40674e854d0ba2d8082af0`.
- `git_status_short_bytes=0`.
- `topology_execution=blocked`.
- `topology_unit_executions=201`.
- `metadata_runs=63`.
- `ring@0.17.14` metadata run: `success`.
- New remaining blocker: `rustc-failed` compiling `crates/crunch-project/src/error.rs`, with `thiserror::Error` derive expansion failing to find `thiserror::__private` and missing `as_dyn_error` on `serde_json::Error`.
- Archive command emitted `cairn/archive/1970-01-01-rust-topology-manifest-links-env/`; it was manually renamed to `cairn/archive/2026-05-28-rust-topology-manifest-links-env/`, then `cairn validate --root .` passed.
