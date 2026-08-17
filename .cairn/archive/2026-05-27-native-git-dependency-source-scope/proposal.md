# Proposal: Native git dependency source scope

## Why

Mantle's current self `rust-plan --execute-topology` probe is still blocked after dependency feature-edge scoping. The remaining package-target blocker chain is:

- `snix-castore` has no ready native package facts.
- `snix-castore` depends on locked git package `wu-manber`.
- native dependency planning reports `unsupported-non-path-dependency` for `wu-manber` even though Cargo's source-closure evidence already records its git URL, revision, package identity, and materialized manifest path.

Without bounded native git source facts, one selected git dependency cascades into native package-target, unit graph, host-unit graph, and topology execution blockers.

## What Changes

- Add a bounded native git dependency source planning scope for Cargo.lock git packages that already appear in the captured source closure.
- Bind each supported git source fact to package identity, git URL, resolved revision, lockfile digest, materialized source root, and deterministic BLAKE3 source-tree digest.
- Allow native dependency edge resolution to use those ready git source facts for selected dependencies such as `snix-castore -> wu-manber`.
- Keep the boundary fail-closed: no network fetch, no version solving, no unbounded `$CARGO_HOME` scanning, and no Cargo git checkout trust beyond the captured source-closure input.

## Impact

- **Files**: `src/rust_plan.rs`, `tests/rust_plan_cli.rs`, `cairn/specs/rust-package-planning/spec.md`
- **Testing**: focused locked-git dependency fixture, missing/stale git source negative fixture, Mantle self probe blocker comparison, Cairn validation and gates.
