## Why

Mantle's Rust package-planning rail now has explicit unit derivation, execution, topology, and output-reuse receipts, but the earliest package/target planning facts are still derived from Cargo oracle material. The next replacement seam is a narrow Mantle-owned planner fragment that can compute package, target, feature, and path-source facts for a small supported workspace, compare them to the retained Cargo oracle, and fail closed before any Cargo-free planning claim when they diverge.

## What Changes

- Add a bounded native Rust package/target planning fragment for simple workspace packages with supported `lib` and `bin` targets.
- Preserve the Cargo oracle as review/comparison evidence while making Mantle-owned planner facts explicit receipt material.
- Add deterministic mismatch and unsupported-shape blockers instead of silently trusting Cargo or falling back to hidden Cargo planning.
- Add CLI/test evidence that proves a tiny supported workspace matches Cargo oracle facts and unsupported package shapes fail closed.

## Impact

- **Files**: `src/rust_plan.rs`, focused Rust-plan fixtures/tests, and `cairn/specs/rust-package-planning/spec.md` after sync.
- **Testing**: focused `rust_plan` tests for supported package/target parity, deterministic mismatch/blocker behavior, Cairn validate, and proposal/design/tasks gates.
