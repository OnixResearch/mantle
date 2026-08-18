## Why

The provider-backed Cargo-free fixed-point proof now progresses past vendored `snix-build` and blocks while compiling the native Mantle binary. The failed unit is local path code and the captured diagnostics are Rust warnings from helper surfaces that are either test-only or intentionally dormant in the current binary.

Mantle should keep local warning hygiene strict enough for provider topology execution without hiding the frontier or weakening source-built closure claims. Test-only helpers should stay scoped to tests, and intentionally dormant feature surfaces should carry scoped allowances so warning noise does not mask the next deterministic provider-proof blocker.

## What Changes

- Add a requirement that provider-backed topology execution must not fail the native Mantle binary on test-only unused-item warnings.
- Move the `BTreeSet` import in `cargo_import` into the test module that uses it.
- Move the musl-target GCC alias constant into the `cargo_free_self_build` test module that uses it.
- Add scoped dormant-code allowances for feature surfaces that are intentionally compiled but not yet wired into the current binary path.
- Rerun focused checks and the provider-backed fixed-point proof to record success or the next deterministic blocker.

## Impact

- **Files**: `src/cargo_import.rs`, `src/cargo_free_self_build.rs`, `src/frontend_artifact_spec.rs`, `src/native_toolchain_closure.rs`, `src/project_build.rs`, `src/rust_bootstrap_patch_plan.rs`, `src/rust_plan.rs`, Cairn specs/evidence.
- **Testing**: baseline receipt extraction, focused unit tests/checks, formatting, Cairn validation/gates, and a real provider-backed fixed-point rerun.
