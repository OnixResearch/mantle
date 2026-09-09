# Proposal: Publish the Wasm component consumer verifier

## Why

Mantle already builds WebAssembly components and emits `mantle-wasm-component-materialization-bundle-v1`. Its pure bundle verifier and file verifier remain internal `crunch-*` implementation surfaces. Independent consumers cannot pin one narrow Mantle-owned verification contract without importing broader build and store authority.

Kamacite records this missing consumer boundary as `external-materialization-verifier-unavailable`. Aspen, Animus, and Lattice also need to remeasure exact bundle members before runtime admission. A published verifier removes that blocker without moving build or product authority into consumers.

## What Changes

- Publish a narrow versioned consumer contract for materialization-bundle decoding, canonical identity validation, required-stage linkage, and ordered blockers. r[mantle.wasm_consumer_verifier.contract]
- Keep structural and canonical-identity decisions in a `no_std + alloc` functional core. r[mantle.wasm_consumer_verifier.functional_core]
- Add a thin standard-library verifier over an explicit capability root and consumer-supplied object resolution. r[mantle.wasm_consumer_verifier.file_shell]
- Emit a bounded verification report that binds the bundle, verified member identities, selected runtime profile, blocker classes, and non-claims. r[mantle.wasm_consumer_verifier.report]
- Publish an immutable source revision and positive and negative consumer fixtures for Kamacite and one additional independent consumer. r[mantle.wasm_consumer_verifier.publication] r[mantle.wasm_consumer_verifier.fixtures]

## Impact

- **Mantle core**: owns bundle syntax, canonical identity, stage linkage, and bounded blocker meaning.
- **Mantle shell**: owns file reads, no-follow capability-relative resolution, byte remeasurement, and report output.
- **Consumers**: retain runtime admission, WIT/world meaning, product policy, authority, effects, receipts, and release decisions.
- **Compatibility**: the existing materialization bundle remains the producer handoff. This change adds a stable consumer surface without changing bundle identity.

## Out of Scope

- Building, composing, virtualizing, precompiling, fetching, or executing components.
- Treating a valid Mantle bundle as component correctness, runtime authority, provenance truth, or release eligibility.
- Adding sibling-path dependencies or ambient store discovery.

## Affected Specs

- `wasm-component-consumer-verifier`: consumer contract, core/shell split, report, publication, fixtures, and non-claims.
