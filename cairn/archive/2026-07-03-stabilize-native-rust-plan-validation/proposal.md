## Why

Native rust-plan focused tests currently provide useful evidence, but some broader validation paths are serial-only or race-prone because fixtures and compiler-policy state can interfere under parallel libtest execution. Mantle needs stable validation so topology hardening can rely on broader checks without special-case human knowledge.

## What Changes

- Identify native rust-plan tests and fixtures that require serial execution today.
- Remove shared mutable fixture state, ambient environment mutation, and output-path collisions where practical.
- Add deterministic fixture isolation for rustc wrappers, build-script outputs, cargo shims, and topology execution directories.
- Preserve tests that intentionally prove ambient-env rejection by running them in subprocesses.

## Impact

- **Files**: `src/rust_plan.rs`, test fixtures, proof scripts, and validation docs.
- **Testing**: repeated serial and parallel focused rust-plan runs, negative ambient-env tests, and topology fixture race repros.

## Out of Scope

- Redesigning native topology planning semantics.
- Making the full workspace clippy/check suite pass through vendored crates.
- Hiding flaky tests by deleting coverage.
