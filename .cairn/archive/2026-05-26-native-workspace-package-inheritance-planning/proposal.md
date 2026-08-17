# Proposal: Native workspace package inheritance planning

## Summary

Support bounded `[workspace.package]` inheritance in native Rust package planning so real Mantle workspace manifests with `version.workspace = true` parse into explicit native facts instead of failing before dependency/source topology work.

## Motivation

Building Mantle with Mantle-owned Rust planning requires the planner to parse Mantle's own workspace manifests. The current parser accepts only literal package versions and therefore blocks before it can evaluate the remaining registry/vendor gaps.

## Scope

- accept literal `package.version = "..."` and bounded `package.version.workspace = true`.
- resolve inherited version from root `[workspace.package].version`.
- emit deterministic blockers for unsupported/missing inheritance material.
- keep Cargo metadata as oracle evidence only.
- add CLI coverage proving inherited package versions become native package facts.

## Non-goals

- full Cargo manifest inheritance semantics.
- inherited dependency metadata beyond the already-modeled bounded seams.
- changing build execution behavior directly.
