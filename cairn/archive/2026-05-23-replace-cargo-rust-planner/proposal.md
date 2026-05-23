## Why

Mantle already owns the derivation, sandbox, store, and receipt pipeline for package builds, but Rust package builds still depend on Cargo as an implicit planner: Cargo resolves features, constructs host/target units, runs build scripts, and synthesizes `rustc` invocations outside Mantle's native derivation graph. That weakens Mantle's reviewability and proof story because important build decisions live in Cargo state rather than in explicit Mantle plans and receipts.

The near-term need is not to remove Cargo in one step. Mantle needs a planned replacement path that first treats Cargo/`cargo build --unit-graph` as an oracle, then incrementally promotes a Mantle-native Rust resolver/planner until Rust packages can be built as explicit Mantle derivations with fail-closed compatibility boundaries.

## What Changes

- Add a Cairn planning package for a Mantle-native Rust package planner that can replace Cargo's orchestration layer over time.
- Define the first-class requirements for:
  - Cargo-oracle parity receipts.
  - Lockfile/source closure ownership.
  - Mantle Rust unit graph planning.
  - Direct `rustc` derivation emission.
  - Host/target split handling for build scripts and proc macros.
  - Fail-closed unsupported Cargo feature boundaries.
- Keep Mantle core generic: language-specific Rust planning belongs in a Rust package frontend/planner layer, not in the core derivation builder.

## Impact

- **Files**: new native Cairn lifecycle root and active change under `cairn/changes/replace-cargo-rust-planner/`, plus repo-local generated Cairn policy for validation.
- **Testing**: `cairn validate --root .` and proposal/design/tasks gates for this planning package.
