# Proposal: Build native Rust unit graph without Cargo unit-graph oracle

## Problem

Mantle still asks Cargo for the selected unit graph. The direct rustc executor can build the resulting nodes, but it is not yet a standalone Cargo alternative because scheduling shape, unit identities, dependency edges, and host/target splits come from Cargo.

## Change

Implement native unit graph construction from Mantle's native manifest, lockfile, source, and feature facts. Keep Cargo comparison as validation when available, but make Cargo-free graph production the claimed path.

## Impact

- **Files**: unit graph planner, host/target partitioning, edge construction, tests, receipt schema.
- **Testing**: oracle comparison suites plus negative unsupported-shape blockers.
