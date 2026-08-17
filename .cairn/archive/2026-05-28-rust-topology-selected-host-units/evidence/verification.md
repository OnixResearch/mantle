# Verification

Task-ID: V1
Covers: rust_package_planning.native_selected_host_units

## Commands

Baseline before code changes:

```sh
env SNIX_BUILD_SANDBOX_SHELL=/nix/store/8mf4s8c4xjvlkj12p299qylrb30g7zzh-busybox-static-x86_64-unknown-linux-musl-1.37.0/bin/busybox CARGO_TARGET_DIR=/tmp/mantle-selected-host-target CARGO_INCREMENTAL=0 nix develop -c cargo test -p mantle --bin mantle native_host_dependencies -- --nocapture
env SNIX_BUILD_SANDBOX_SHELL=/nix/store/8mf4s8c4xjvlkj12p299qylrb30g7zzh-busybox-static-x86_64-unknown-linux-musl-1.37.0/bin/busybox CARGO_TARGET_DIR=/tmp/mantle-selected-host-target CARGO_INCREMENTAL=0 nix develop -c cargo test -p mantle --bin mantle combined_unit_topology_orders_host_build_script_before_same_package_proc_macro -- --nocapture
```

Post-change focused tests:

```sh
nix develop -c rustfmt src/rust_plan.rs
env SNIX_BUILD_SANDBOX_SHELL=/nix/store/8mf4s8c4xjvlkj12p299qylrb30g7zzh-busybox-static-x86_64-unknown-linux-musl-1.37.0/bin/busybox CARGO_TARGET_DIR=/tmp/mantle-selected-host-target CARGO_INCREMENTAL=0 nix develop -c cargo test -p mantle --bin mantle native_host_planning -- --nocapture
env SNIX_BUILD_SANDBOX_SHELL=/nix/store/8mf4s8c4xjvlkj12p299qylrb30g7zzh-busybox-static-x86_64-unknown-linux-musl-1.37.0/bin/busybox CARGO_TARGET_DIR=/tmp/mantle-selected-host-target CARGO_INCREMENTAL=0 nix develop -c cargo test -p mantle --bin mantle native_host_dependencies -- --nocapture
env SNIX_BUILD_SANDBOX_SHELL=/nix/store/8mf4s8c4xjvlkj12p299qylrb30g7zzh-busybox-static-x86_64-unknown-linux-musl-1.37.0/bin/busybox CARGO_TARGET_DIR=/tmp/mantle-selected-host-target CARGO_INCREMENTAL=0 nix develop -c cargo test -p mantle --bin mantle combined_unit_topology_orders_host_build_script_before_same_package_proc_macro -- --nocapture
```

Dirty self-probe:

```sh
pueue task 64: cargo run -q -p mantle --bin mantle -- --json rust-plan --root . --cargo /home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/cargo --rustc /home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/rustc --target x86_64-unknown-linux-gnu --execute-topology --execution-output-root target/mantle-self-rust-plan-probe-selected-host-dirty3/execution
```

## Results

- Baseline host dependency tests passed: `2 passed` and `1 passed`.
- Post-change `native_host_planning`: `2 passed`.
- Post-change `native_host_dependencies`: `2 passed`.
- Post-change same-package topology test: `1 passed`.
- Dirty self-probe receipt: `target/mantle-self-rust-plan-probe-selected-host-dirty3/receipt.json`.
- Dirty self-probe status: `topology_execution=blocked`, `topology_unit_executions=152`, `metadata_runs=44`.
- `jiff-static` executions: none.
- `aws-lc-sys` and `aws-lc-rs` metadata runs: `success`.
- Previous unselected `jiff-static` unresolved `quote`/`syn` blocker is absent.
- New remaining blocker: `build-script-run-failed` in `vendor-deps/ring/build.rs`, panic at line 287 from `Option::unwrap()` on `None`.

Clean self-probe:

```sh
pueue task 65: cargo run -q -p mantle --bin mantle -- --json rust-plan --root . --cargo /home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/cargo --rustc /home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/rustc --target x86_64-unknown-linux-gnu --execute-topology --execution-output-root target/mantle-self-rust-plan-probe-after-21bd9c17-clean/execution
```

Clean self-probe result:

- Receipt: `target/mantle-self-rust-plan-probe-after-21bd9c17-clean/receipt.json`.
- HEAD: `21bd9c174225d136131c10a12494384d541b291e`.
- `git_status_short_bytes=0`.
- `topology_execution=blocked`.
- `topology_unit_executions=152`.
- `metadata_runs=44`.
- `jiff-static` executions: none.
- `aws-lc-sys` and `aws-lc-rs` metadata runs: `success`.
- New remaining blocker: `build-script-run-failed` in `vendor-deps/ring/build.rs`, panic at line 287 from `Option::unwrap()` on `None`.
- Archive command emitted `cairn/archive/1970-01-01-rust-topology-selected-host-units/`; it was manually renamed to `cairn/archive/2026-05-28-rust-topology-selected-host-units/`, then `cairn validate --root .` passed.
