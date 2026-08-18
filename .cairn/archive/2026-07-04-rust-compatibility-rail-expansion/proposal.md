## Why

The representative Rust compatibility rail is intentionally small: one binary, one local library, one vendored registry source, one proc macro, and one simple build script metadata surface. That rail is useful for guarding non-overclaiming, but it does not exercise many practical offline Rust workspace surfaces that fail in real projects: feature resolution, target-specific dependencies, multiple packages/binaries, linked native libraries, `pkg-config`, build-script link metadata, vendored git sources, and unsupported boundaries.

Mantle should expand the rail as a surface matrix rather than as a vague “full Cargo compatibility” claim. Each surface should have positive coverage when supported and negative/fail-closed coverage when unsupported, with separate evidence classes for sandboxed offline Cargo and native `rust-plan`.

## What Changes

- Add a source-controlled Rust compatibility surface matrix that names supported, blocked, and deliberately out-of-scope Cargo/Rust surfaces.
- Expand the representative fixtures to cover target cfg/features, workspace inheritance, multiple packages/binaries, vendored registry/git sources, proc macros, build-script env/cfg/link metadata, `links`, `pkg-config`, and small native C compilation where supported.
- Keep the offline Cargo rail and native `rust-plan` rail separate in reports and docs.
- Require deterministic blockers for unsupported surfaces rather than hidden Cargo fallback or broad compatibility claims.
- Update examples/docs so the rail communicates frontier movement without implying full Cargo compatibility.

## Impact

- **Files**: `tests/rust_compatibility_rail.rs`, `examples/rust_compatibility_rail.rs`, `examples/catalog.ncl`, `examples/README.md`, `src/rust_plan.rs`, `src/cargo_import.rs`, docs, and this Cairn spec delta.
- **Testing**: positive fixture expansion, negative unsupported-surface fixtures, no-Cargo fallback guard, offline Cargo smoke, rust-plan receipt classification, and docs/catalog drift checks.

## Out of Scope

- Claiming complete Cargo compatibility.
- Executing full crates.io-scale compatibility testing.
- Replacing the bounded offline Cargo project-build lane as the default build path.
- Proving rustc/compiler correctness or release reproducibility.
