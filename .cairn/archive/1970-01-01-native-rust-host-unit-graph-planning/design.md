## Overview

This change defines the planning boundary for native Rust host-unit graph facts. The implementation remains bounded to reviewable package shapes that Mantle can already parse natively: local/path workspaces, supported source closures, normal lib/bin targets, and the first host-unit surfaces required by Cargo-free topology execution.

Cargo remains present only as an oracle. Mantle-owned native facts must drive the supported host-unit graph receipt and the downstream derivation graph; Cargo unit-graph material is retained so the receipt can prove equivalence or explain a deterministic mismatch.

## Data Model

Add a host-unit planning receipt section such as `native_host_unit_graph_planning` with:

- `ready`: true only when all supported host-unit facts compare cleanly against the Cargo oracle.
- `native_host_graph_digest`: BLAKE3 digest of canonical native host unit, artifact, and consumer-edge facts.
- `cargo_host_oracle_digest`: BLAKE3 digest of the normalized Cargo unit-graph host-unit subset used for comparison.
- `oracle_comparison_digest`: BLAKE3 digest of the deterministic comparison inputs/outcome.
- `host_units`: ordered bounded facts for `custom-build` and `proc-macro` units, including package identity, target identity, host execution kind, source inputs, declared artifact class, and generated-metadata placeholder surfaces.
- `target_consumers`: ordered edges from target units to consumed host artifacts/metadata.
- `blockers`: deterministic issue records for unsupported or mismatched inputs.
- `receipt_hash`: self-reference-safe BLAKE3 hash of the section.

## Supported Fragment

The first supported fragment should include:

- local/path workspace packages already accepted by native package/target planning;
- `build.rs` represented as `custom-build` host units;
- `proc-macro = true` library targets represented as `proc-macro` host units;
- target lib/bin units consuming build-script artifacts/metadata or proc-macro artifacts;
- build mode only, default features only unless the existing native planner has explicit support for the requested feature surface.

Anything outside that fragment must block readiness rather than silently falling back to Cargo.

## Derivation Integration

When `native_host_unit_graph_planning.ready == true`, `unit_derivation_graph` must consume the native host-unit facts for host nodes and target host-artifact inputs. Cargo unit-graph facts may stay in the receipt only as oracle evidence and must not be the source of supported host/target topology facts.

Host unit derivation summaries should keep existing execution-receipt contracts: host units produce explicit artifacts or metadata placeholders; target consumers list consumed host artifacts and metadata surfaces; host units do not consume their own reserved artifacts.

## Fail-Closed Boundaries

Blockers should cover at least:

- native package/target or native unit graph planning not ready;
- missing or unreadable source/manifest facts required for a host target;
- unsupported target kinds, unit modes, profiles, feature surfaces, or dependency kinds;
- unresolved, ambiguous, or host/target-confused consumer edges;
- missing build-script or proc-macro target facts;
- native host graph disagreement with the Cargo oracle.

## Verification Plan

- Positive focused test: a supported workspace with a build script and/or proc macro produces ready native host-unit graph facts, compares cleanly against Cargo oracle material, and feeds host nodes plus target consumer edges into `unit_derivation_graph`.
- Negative focused tests: unsupported host surface, unresolved consumer edge, missing native package/source facts, and native-vs-oracle mismatch each produce deterministic blockers with `ready=false`.
- Receipt checks: hashes are non-empty, self-reference-safe, sorted/canonical, and distinguish Mantle-owned host facts from Cargo oracle evidence.
- Lifecycle checks: `cairn validate --root .`, `gate proposal`, `gate design`, and `gate tasks` pass before implementation or archive.
