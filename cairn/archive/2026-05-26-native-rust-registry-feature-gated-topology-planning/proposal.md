# Native Rust registry feature-gated topology planning

## Summary

Add a bounded native planning slice for vendored registry dependency graphs whose shape depends on explicit Cargo feature declarations.

Mantle now executes registry-backed leaf, host-artifact, proc-macro, and transitive registry topologies from ready vendored source facts. The next realism gap is feature-gated registry surfaces: real crates commonly use optional dependencies and selected feature sets to decide which registry units enter the graph.

This change should make Mantle's behavior explicit at that seam: either plan and execute a narrow supported feature-selected vendored registry topology from reviewable native facts, or fail closed before `rustc` with deterministic blockers when feature resolution would require hidden Cargo resolver behavior.

## Motivation

The accepted Rust package-planning spec still treats unsupported feature surfaces as blockers. That is correct for broad Cargo compatibility, but Mantle needs the first explicit feature seam so future registry work does not silently depend on Cargo's resolver, ambient metadata, or accidental default-feature behavior.

A narrow feature-gated registry slice lets Mantle prove the shape of feature-selected packages and dependency artifacts while preserving no Cargo orchestration, no network/index access, no `$CARGO_HOME`, no registry cache fallback, and no version solving.

## Scope

- Start with a bounded vendored registry graph where a selected feature enables one optional registry dependency.
- Represent selected features, optional dependency activation, and resulting unit graph evidence in native planning receipts.
- Gate execution on ready native registry source facts for all selected registry packages.
- Prove positive CLI JSON evidence for the supported feature-selected graph.
- Prove negative CLI JSON evidence for unsupported feature surfaces before any affected `rustc` invocation.

## Non-goals

- No full Cargo resolver compatibility.
- No feature unification across arbitrary workspaces.
- No version solving.
- No network/index fetch.
- No Cargo build orchestration.
- No `$CARGO_HOME` or ambient registry cache fallback.
- No broad default-feature semantics beyond the explicitly supported fixture.
