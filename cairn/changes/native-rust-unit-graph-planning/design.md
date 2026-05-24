# Design: Native Rust Unit Graph Planning

## Current state

`rust-plan` currently captures:

1. Cargo metadata and Cargo unit graph oracle material.
2. Mantle-owned source closure facts.
3. Mantle-owned native package/target planning facts for bounded local/path workspaces.
4. `unit_derivation_graph` facts derived from Cargo's `cargo build --unit-graph` JSON.

The remaining hidden dependency is step 4: even for a supported local/path package graph, unit identities, dependency edges, modes, profiles, and target-unit shape originate from Cargo's unit graph.

## Proposed native fragment

Add a native unit graph planning summary adjacent to the existing native package/target summary. The native fragment should be computed from Mantle-owned package/target/source facts plus selected CLI options, not from Cargo unit graph JSON.

For the first slice, support:

- workspace members already accepted by `native_package_target_planning`;
- local/path dependencies whose package identity appears in the native package facts;
- normal target units for `lib` and `bin` targets;
- build mode only;
- enough profile/target/source/dependency facts to produce the same existing `UnitDerivationSummary` shape for supported nodes.

Cargo unit graph remains captured as an oracle. The native fragment should compute a canonical digest for native unit nodes/edges and an oracle comparison digest. If native and oracle facts disagree, the fragment is not ready and records deterministic mismatch blockers.

## Fail-closed boundaries

The native fragment must not silently fall back to Cargo unit facts. It should emit deterministic blockers for:

- native package/target planning not ready;
- unsupported target kinds or unit modes;
- unsupported feature resolver surfaces or non-default feature requests beyond the supported fragment;
- dependency edges that cannot be resolved through local/path native package facts;
- source-closure blockers that make unit source material incomplete;
- native-vs-oracle unit identity, target, profile, mode, source, or dependency-edge mismatch.

## Receipt flow

The receipt should make the ownership boundary visible:

- Cargo unit graph digest: retained oracle evidence.
- Native unit graph digest: Mantle-owned unit facts.
- Oracle comparison digest: deterministic comparison evidence.
- `ready`: true only when the native fragment is supported and matches the oracle.
- `blockers`: deterministic blocker list when unsupported or mismatched.

Once ready, `unit_derivation_graph` should be able to consume native unit graph facts for the supported fragment. Existing Cargo-derived behavior can remain available only as oracle material and for unsupported non-claim receipts, not as a hidden success path for native claims.

## Test strategy

Add focused unit tests in `src/rust_plan.rs`:

- positive tiny path workspace with producer lib and consumer lib/bin target where native unit graph facts match the Cargo-oracle-shaped fixture and yield ready derivation nodes;
- negative unsupported target/mode/feature surfaces;
- negative mismatch fixture proving native-vs-oracle drift blocks readiness;
- negative missing path dependency/native package fact fixture.

Keep tests fixture-driven and deterministic; do not require network or registry access.
