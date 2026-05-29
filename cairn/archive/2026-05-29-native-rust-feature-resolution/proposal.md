# Proposal: Resolve Rust features natively

## Problem

Mantle currently inherits Cargo-selected features from Cargo unit graph material. A Cargo-free planner must compute feature activation itself, including resolver behavior, optional dependencies, defaults, and target/build/dev partitioning.

## Change

Implement a native feature resolver for the bounded Cargo subset Mantle claims. Emit selected features per package/unit plus deterministic blockers for unsupported feature surfaces.

## Impact

- **Files**: feature resolver core, manifest model, unit graph lowering, fixtures.
- **Testing**: resolver v2 fixtures, optional dependency negative cases, Cargo oracle comparison.
