# Proposal: Plan Rust packages from manifests and lockfiles without Cargo metadata

## Problem

Mantle still invokes Cargo to discover workspace membership, package metadata, target metadata, dependency edges, and lockfile identities. That keeps native execution Cargo-guided even when rustc execution is Cargo-free.

## Change

Implement a native manifest and lockfile planner that reads `Cargo.toml`, workspace configuration, `.cargo/config` inputs we choose to support, and `Cargo.lock` directly. The planner emits reviewable package/source/target facts without calling Cargo.

## Impact

- **Files**: Rust planning parser/model code, CLI options, tests, fixtures.
- **Testing**: positive/negative manifest and lockfile fixtures, oracle comparison while Cargo remains available.
