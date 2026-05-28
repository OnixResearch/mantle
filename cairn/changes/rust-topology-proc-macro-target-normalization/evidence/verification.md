# Verification

Focused tests:

```sh
export PATH=/home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin:/nix/store/97vplpbajnr7x03fqh9biz5v6960sv22-clang-wrapper-21.1.8/bin:/nix/store/1sw8whfl5gfblp6r9qdkiw1b4j9fgwar-mold-2.40.4/bin:/nix/store/rvp7qlpf5jqvdckjy1afjb6aha6j8dxg-pkg-config-wrapper-0.29.2/bin:$PATH
export PKG_CONFIG_PATH=/nix/store/1l5jgzy26hkjz1y3apn1051asvn42sfn-openssl-3.6.1-dev/lib/pkgconfig
export SNIX_BUILD_SANDBOX_SHELL=/nix/store/8mf4s8c4xjvlkj12p299qylrb30g7zzh-busybox-static-x86_64-unknown-linux-musl-1.37.0/bin/busybox
export CARGO_TARGET_DIR=/tmp/mantle-proc-macro-target-normalization-test
cargo test -p mantle --bin mantle native_host_planning_normalizes_proc_macro_target_names_and_follows_selected_units_only -- --nocapture
cargo test -p mantle --bin mantle native_host_planning_keeps_selected_same_package_build_script_for_proc_macro -- --nocapture
cargo test -p mantle --bin mantle native_host_planning -- --nocapture
```

Results:

- `native_host_planning_normalizes_proc_macro_target_names_and_follows_selected_units_only`: `1 passed`.
- `native_host_planning_keeps_selected_same_package_build_script_for_proc_macro`: `1 passed`.
- `native_host_planning`: `2 passed`.

Dirty self-probe:

```sh
pueue task 74: cargo run -q -p mantle --bin mantle -- --json rust-plan --root . --cargo /home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/cargo --rustc /home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/rustc --target x86_64-unknown-linux-gnu --execute-topology --execution-output-root target/mantle-self-rust-plan-probe-proc-macro-target-dirty/execution
```

Dirty self-probe result:

- Probe root: `target/mantle-self-rust-plan-probe-proc-macro-target-dirty/`.
- HEAD before implementation commit: `1c0ad2e0c0f3d2a1a8a85ac68556634ebbf8a73f`.
- `git_status_short_bytes=38`.
- Probe status: `3`.
- `curve25519-dalek-derive@0.1.1` proc macro execution: `success` (`execution/533_.../.mantle-rust-unit-execution.json`).
- `curve25519-dalek@4.1.3` custom-build execution: `success` (`execution/534_.../.mantle-rust-unit-execution.json`).
- `curve25519-dalek@4.1.3` lib execution: `success` (`execution/113_.../.mantle-rust-unit-execution.json`).
- Previous curve25519 duplicate-import blocker is absent.
- New remaining frontier: internal execution error before JSON receipt: `unit 62:registry+https://github.com/rust-lang/crates.io-index#aws-lc-sys@0.39.1:aws_lc_sys:lib:build reached execution before dependency package registry+https://github.com/rust-lang/crates.io-index#aws-lc-sys@0.39.1 was produced`.

Clean post-commit self-probe pending.
