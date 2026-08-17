# Proposal: Native registry transitive producer coverage

## Why

After bounded git source facts landed, the current Mantle self `rust-plan --execute-topology` probe has one remaining blocker class:

- `missing-dependency-producer`: `no supported target producer lib unit for dependency package registry+https://github.com/rust-lang/crates.io-index#itertools@0.10.5`

The native package-target planner already records supported package facts for `itertools@0.10.5`, including a vendored manifest and `lib` target. The later topology execution still cannot find a producer unit for the dependency artifact. That means the native graph can claim readiness while a supported transitive registry dependency is not covered by an executable producer unit.

## What Changes

- Extend native unit/derivation graph planning so every consumed dependency artifact for a supported package has an eligible producer unit when matching native package, source, and `lib` target facts exist.
- Treat transitive registry packages the same as path and captured git packages once their source facts are ready and BLAKE3-bound.
- Move unsupported or missing producer coverage into deterministic planning blockers before topology execution claims readiness.
- Preserve bounded behavior: no Cargo orchestration, no ambient registry/git cache lookup, no version solving, and no execution from undeclared source material.

## Impact

- **Files**: `src/rust_plan.rs`, focused tests in `tests/rust_plan_cli.rs` or unit tests near native unit graph/topology planning, `cairn/specs/rust-package-planning/spec.md`
- **Testing**: focused fixture for a transitive vendored registry dependency producer, negative fixture for an unresolved producer, Mantle self probe blocker movement, Cairn validation and gates.
