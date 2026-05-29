# Proposal: Expose Cargo-free Rust planning and build CLI

## Problem

Today the practical path still runs through `cargo run` and Cargo oracle flags. Users need an explicit product surface that builds with Mantle's native planner/executor and refuses to invoke Cargo when Cargo-free mode is requested.

## Change

Add a CLI mode for native Rust planning/building without Cargo oracle dependencies. The mode should report clear claims, blockers, and non-claims, and should make Cargo use impossible or audited.

## Impact

- **Files**: CLI flags/commands, execution mode plumbing, JSON receipt schema, docs/tests.
- **Testing**: CLI positive and negative tests with Cargo removed from PATH or replaced by a failing shim.
