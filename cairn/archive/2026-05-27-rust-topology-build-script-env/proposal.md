## Why

After native edition propagation, topology execution reaches the first build script and fails because Mantle runs the compiled build-script binary with a cleared environment that lacks `$RUSTC`. Cargo build scripts are not ordinary binaries: even in Mantle's bounded topology rail, they need a small deterministic execution environment and package-root working directory to emit metadata.

## What Changes

- Add a bounded build-script child environment derived from `RustUnitExecutionOptions` and the selected unit.
- Set `RUSTC`, `HOST`, `TARGET`, and `PROFILE` for build-script execution while preserving existing `OUT_DIR`, `CARGO_MANIFEST_DIR`, and `CARGO_PKG_NAME` behavior.
- Run build scripts with current directory set to the package manifest directory.
- Add focused tests for positive env/cwd binding and negative missing-source behavior.
- Re-run the self-probe to verify movement past the missing `$RUSTC` blocker.

## Impact

- **Files**: `src/rust_plan.rs`, `cairn/specs/rust-package-planning/spec.md`, change evidence/tasks.
- **Testing**: focused build-script env tests, clean self-probe, `cairn validate`, `git diff --check`.
