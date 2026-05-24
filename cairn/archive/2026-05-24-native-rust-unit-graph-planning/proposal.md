# Proposal: Native Rust Unit Graph Planning

## Summary

Introduce the next bounded Cargo-replacement seam after native package/target planning: Mantle-owned construction of a supported Rust unit graph fragment for tiny local/path workspaces, while retaining Cargo's `cargo build --unit-graph` output only as oracle comparison evidence.

## Why

Mantle now computes native package/target/source facts and compares them against Cargo metadata, but `rust-plan` still derives `unit_derivation_graph` from Cargo's unit graph. That keeps Cargo as the hidden source of earliest build-unit edges even when package/target facts are already Mantle-owned.

The next high-ROI step is to build the first native unit graph fragment from Mantle's package/target facts, compare it against Cargo's oracle unit graph for supported shapes, and fail closed for unsupported or mismatched cases.

## Scope

- Supported shape: tiny local/path workspaces, normal `lib` and `bin` targets, path dependencies already represented by the native package/target planner, and default build profile facts needed by existing derivation receipts.
- Preserve Cargo oracle output as comparison evidence, not as the source of native unit facts.
- Emit deterministic blockers for unsupported target kinds, unsupported dependency kinds/features, missing native package facts, missing or ambiguous dependency edges, and native-vs-oracle mismatches.
- Feed the existing `unit_derivation_graph` receipt path from native unit graph facts only when the fragment is ready.

## Non-goals

- Full Cargo compatibility.
- Test/doctest/example/bench planning.
- Feature resolver parity beyond the bounded default/no-feature fragment.
- Build-script/proc-macro scheduling beyond existing explicit host-artifact receipt boundaries.
- Removing Cargo oracle capture entirely.
