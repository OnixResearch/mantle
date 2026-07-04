# Proposal: Schema contract generation rail

## Summary

Adopt the `json-schema-to-nickel` pattern for Mantle's machine-readable surfaces: JSON schemas and generated Nickel contracts should stay in sync, with positive and negative fixtures proving that report contracts accept valid payloads and reject malformed ones.

## Motivation

Mantle emits many JSON reports and receipts: build reports, doctor output, proof manifests, demo summaries, release evidence, and project diagnostics. These surfaces are useful only if their schemas are stable and validated. `json-schema-to-nickel` provides the key adoption pattern: generate Nickel contracts from JSON Schema where possible, keep lazy/inspectable contracts when available, and test both generated text and runtime validation behavior.

Mantle should use generated contracts as a development and release rail, not as a runtime dependency on arbitrary schema conversion.

## Scope

- Inventory Mantle's documented JSON surfaces and identify the schema owner for each.
- Add a deterministic schema-to-Nickel contract generation/check workflow.
- Commit generated contracts or generated-contract snapshots for machine surfaces selected by the change.
- Add positive and negative fixture validation for generated contracts.
- Label cases where schema conversion falls back to opaque predicate validation or unsupported JSON Schema features.

## Non-goals

- No runtime execution of arbitrary remote JSON Schema conversion during normal Mantle commands.
- No claim that schema validation proves build correctness, release reproducibility, or deployability.
- No requirement to support every JSON Schema dialect; unsupported dialects must be explicit blockers.
- No replacement for Rust type-level validation in functional cores.

## Target Spec Domains

- `verification-evidence` for proof-before-claim rules around machine-schema contract freshness and validation evidence.
