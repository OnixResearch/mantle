## Why

After `CARGO_MANIFEST_LINKS` parity, native topology reaches `crates/crunch-project` and fails inside `thiserror::Error` expansion. The generated code references the `thiserror` 1.x private API while the target crate links `thiserror` 2.x. Mantle resolves registry dependency sources by package name only, so same-name multi-version packages can bind to the wrong vendored manifest.

## What Changes

- Make native registry dependency source resolution version-aware.
- Preserve deterministic registry fallback, but prefer an exact or dotted-prefix version match when same-name candidates exist and fail closed when an exact version is missing.
- Add focused positive and negative tests for same-name registry packages.

## Impact

- **Files**: `src/rust_plan.rs`, `cairn/specs/rust-package-planning/spec.md`
- **Testing**: focused registry dependency tests, clean self-probe, `cairn validate`, Cairn gates.
