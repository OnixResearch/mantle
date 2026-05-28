## Why

The native build-script environment now binds deterministic Cargo-like variables, but `CARGO_PKG_NAME` is still derived from the build-script target name. Cargo sets `CARGO_PKG_NAME` to the manifest package name, preserving hyphens. Build scripts that inspect package identity should not see `build_script_build` for every custom-build target.

## What Changes

- Carry the native package name into generated build-script unit derivation environment.
- Set `CARGO_PKG_NAME` from package-derived data, not the build-script target name.
- Keep the bounded environment fallback deterministic for legacy/Cargo-oracle units.
- Add focused positive and fallback tests.
- Re-run the self-probe to verify the topology frontier does not regress.

## Impact

- **Files**: `src/rust_plan.rs`, `cairn/specs/rust-package-planning/spec.md`, change evidence/tasks.
- **Testing**: focused build-script env tests, clean self-probe, `cairn validate`, `git diff --check`.
