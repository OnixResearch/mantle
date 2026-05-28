## Why

The native Rust topology now compiles and runs the `aws-lc-sys` build script after package metadata env parity, but execution stops because Cargo target cfg runtime variables are absent. The current clean probe shows `aws-lc-sys` panics on missing `CARGO_CFG_TARGET_ARCH`.

## What Changes

- Derive a bounded Cargo-compatible target cfg env surface from Mantle's active target triple for native build-script child execution.
- Populate deterministic `CARGO_CFG_TARGET_*` variables such as arch, vendor, os, env, family, endian, and pointer width.
- Keep unsupported target triples explicit by leaving unknown components deterministic instead of reading ambient Cargo state.
- Record whether the self-probe moves past the missing `CARGO_CFG_TARGET_ARCH` blocker.

## Impact

- **Files**: `src/rust_plan.rs`, Cairn change/evidence files.
- **Testing**: focused `rust_plan::` tests, clean self-probe, `cairn validate --root .`, `git diff --check`.
