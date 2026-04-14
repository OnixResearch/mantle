# Reduce build pipeline overhead

## Why

Crunch spends too much time before and after real build execution on several
repeatable hot paths:

- the pipeline still exports the whole Nickel result to JSON and reparses it
  before conversion,
- source closure walks and source ingestion repeat across derivations in the
  same build session,
- remote closure fallback can fetch full NAR payloads just to discover
  `References:` metadata,
- worker dispatch and completion still clone large derivation and build-request
  payloads on the hot path.

Those costs are most visible in self-build and multi-root package-set builds,
but they also slow ordinary iterative development.

## What Changes

- switch the build pipeline from JSON-based derivation extraction to direct
  typed Nickel deserialization for `CrunchDerivation`-shaped data
- memoize source closure expansion and reusable input-node materialization
  within a build session
- split remote closure metadata lookup from full binary-cache substitution so
  closure walks can use narinfo references without downloading content
- reduce worker hot-path clone churn in ready, dispatch, and completion paths

## Capabilities

### New Capabilities

- `direct-typed-derivation-extraction`: build execution can consume evaluated
  Nickel derivations without a whole-program JSON round-trip
- `session-source-resolution-cache`: repeated source inputs in one build
  session reuse closure and node-resolution work
- `metadata-only-remote-closure-lookup`: remote closure walks can read narinfo
  references without substituting the corresponding NAR payload

## Impact

- **Files**: `crates/crunch-pipeline/src/lib.rs`, `crates/crunch-eval/src/lib.rs`,
  `crates/crunch-build/src/worker.rs`, `crates/crunch-build/src/orchestrate.rs`,
  and remote-store lookup code under `crates/crunch-store/` or vendored
  `snix-store`
- **Behavior**: build preparation becomes cheaper without changing derivation
  semantics or result reporting
- **Testing**: add pipeline, store, and worker regression coverage for direct
  extraction, source-resolution reuse, and metadata-only closure lookups

## Non-Goals

- add cargo dependency-layer caching or other self-build artifact caching yet
- parallelize `prepare_build()` and `finish_build()` in this change
- redesign fetcher transport or binary-cache verification semantics
