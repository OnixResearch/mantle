## Why

After selected host-unit planning, the clean self-probe advances to `ring`'s build script. `vendor-deps/ring/build.rs` unwraps `CARGO_MANIFEST_LINKS` and asserts it matches the package `links` value. Mantle currently provides bounded `CARGO_PKG_*` package metadata but omits `CARGO_MANIFEST_LINKS`, so linked build scripts can panic before emitting metadata.

## What Changes

- Add deterministic `CARGO_MANIFEST_LINKS` to native build-script package env.
- Set it to the manifest `[package].links` value when present, else empty string.
- Add focused positive and negative tests for linked and unlinked packages.

## Impact

- **Files**: `src/rust_plan.rs`, `cairn/specs/rust-package-planning/spec.md`
- **Testing**: focused package env tests, clean self-probe, `cairn validate`, Cairn gates.
