## Why

Mantle must consume the published product-neutral reconciliation contract through an immutable revision before any storage-uncertainty adoption. A pilot proves the composition without transferring GC meaning into the shared core.

## What Changes

- Pin `transactional-reconciliation-core` at Radicle revision `eb2bd3441753af97bfcb247cef7cc22d72675b62` in both Cargo and the Nix flake.
- Add one product-boundary pilot test that binds an exact GC plan identity through the shared planner and classifies a changed plan as stale.

## Impact

- **Files**: `Cargo.toml`, `Cargo.lock`, `flake.nix`, `flake.lock`, `README.md`, `tests/transactional_reconciliation_pilot.rs`, import ordering in `src/source_built_fixed_point_shell.rs`
- **Testing**: focused pilot test, scoped first-party quality and tigerstyle checks on the touched crate
