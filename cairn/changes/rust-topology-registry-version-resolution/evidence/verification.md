# Verification

Baseline before implementation:

```sh
cargo test -p mantle --bin mantle native_registry_source_planning_binds_declared_vendor_source -- --nocapture
```

First attempt without documented tool PATH failed before tests with `linker 'clang' not found`. Rerun with documented clang/mold/pkg-config PATH, `PKG_CONFIG_PATH`, `SNIX_BUILD_SANDBOX_SHELL`, and isolated `CARGO_TARGET_DIR=/tmp/mantle-registry-version-target` passed:

- `native_registry_source_planning_binds_declared_vendor_source`: `1 passed`.

Focused post-change tests with documented tool PATH and isolated `CARGO_TARGET_DIR=/tmp/mantle-registry-version-target`:

```sh
cargo test -p mantle --bin mantle registry_dependency_source -- --nocapture
cargo test -p mantle --bin mantle native_registry_source_planning_binds_declared_vendor_source -- --nocapture
cargo test -p mantle --bin mantle native_host_planning -- --nocapture
```

Results:

- `registry_dependency_source`: `2 passed`.
- `native_registry_source_planning_binds_declared_vendor_source`: `1 passed`.
- `native_host_planning`: `2 passed`.

Dirty self-probe:

```sh
pueue task 71: cargo run -q -p mantle --bin mantle -- --json rust-plan --root . --cargo /home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/cargo --rustc /home/brittonr/.rustup/toolchains/nightly-x86_64-unknown-linux-gnu/bin/rustc --target x86_64-unknown-linux-gnu --execute-topology --execution-output-root target/mantle-self-rust-plan-probe-registry-version-dirty3/execution
```

Dirty self-probe result:

- Receipt: `target/mantle-self-rust-plan-probe-registry-version-dirty3/receipt.json`.
- HEAD: `f05218d5eb05bfe116e6e83792804d259213f323`.
- `git_status_short_bytes=38`.
- `topology_execution=blocked`.
- `topology_unit_executions=217`.
- `metadata_runs=63`.
- `thiserror@2.0.18` consumed host artifacts now include `thiserror-impl@2.0.18` proc macro and its own custom build artifact.
- `thiserror@2.0.18` lib execution status: `success`.
- New remaining blocker: `rustc-failed` compiling `vendor-deps/curve25519-dalek/src/backend/vector/scalar_mul/variable_base.rs`, with duplicate `CachedPoint` / `ExtendedPoint` imports from both `avx2` and `ifma` cfg paths.

Clean post-commit self-probe and archive validation are pending until the implementation commit exists.
