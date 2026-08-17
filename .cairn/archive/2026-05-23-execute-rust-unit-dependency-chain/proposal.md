## Why

Mantle can now execute one supported `lib`/`bin` unit from an explicit `unit_derivation_graph` and emit a deterministic execution receipt. The next orchestration seam is a dependency edge: a consuming Rust unit should use an artifact produced by a prior explicit unit, with no Cargo target directory or cache discovery.

This change moves the bounded claim from an isolated unit to a two-unit dependency chain while keeping the scope narrow and fail-closed.

## What Changes

- Add a bounded execution rail for one supported target unit that consumes one or more explicit dependency artifacts produced by earlier supported `lib`/`bin` units in the same `unit_derivation_graph`.
- Record per-unit execution receipts for both the dependency producer and the consuming unit.
- Rewrite only the declared dependency artifact placeholders to the produced artifact paths; do not discover artifacts from Cargo caches or ambient target directories.
- Add blockers for missing producer units, missing produced artifacts, stale/unreadable dependency artifacts, graph blockers, host-artifact requirements, or unsupported multi-step shapes.

## Impact

- **Files**: expected changes in `src/rust_plan.rs` and focused tests; CLI exposure is optional unless needed for evidence.
- **Testing**: positive two-unit dependency-chain fixture, negative stale/missing dependency artifact fixture, focused `rust_plan` tests, Cairn validate/gate, and `git diff --check`.
