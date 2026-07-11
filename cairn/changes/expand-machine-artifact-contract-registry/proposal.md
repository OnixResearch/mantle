## Why

Mantle has a sound pilot for Nickel-backed machine artifacts: `schemas/machine-contracts/inventory.ncl` registers `operator-diagnostics.doctor-report`, a JSON Schema owns the public shape, a generated Nickel contract mirrors it, positive and negative fixtures exercise it, and `scripts/check-machine-schema-contracts.rs` checks freshness.

The pilot currently covers one report while Mantle exposes many stable machine-readable build, plan, routing, source-bundle, portable-receipt, release, attestation, import/export, and remote-operation DTOs. Those surfaces are versioned and consumed by scripts or sibling stack components, but most have no registry classification, generated Nickel review contract, fixture contract, or common unsupported-version policy. The checker itself is consequently hard-coded to the doctor report rather than data-driven.

## What Changes

- Expand the machine-artifact inventory so every public machine JSON surface is classified as contracted, internal/debug, compatibility-only, or externally owned, with an owner and rationale.
- Turn the one-surface checker into a data-driven registry rail that verifies schema, generated Nickel contract, fixture, consumer, version-policy, non-claim, and BLAKE3 identity fields.
- Add generated Nickel contracts for an initial high-value cohort spanning build/plan, realization routing, portable receipts, source bundles, Nickel export receipts, and release/attestation handoff.
- Derive or deterministically render review schemas from Rust-owned runtime DTOs and prove serialization parity so the schema and Rust representation cannot silently diverge.
- Require positive and negative fixtures for exact schema/version, enums, bounds, BLAKE3 fields, safe references, redaction, unknown fields, and cross-field invariants.
- Keep report production and runtime validation in Rust; Nickel remains a review/consumer boundary and is never evaluated in build or release hot paths.

## Impact

- **Existing pilot**: `schemas/machine-contracts/` and `scripts/check-machine-schema-contracts.rs` become the reusable registry/generation rail.
- **Initial Rust owners**: `src/build_report.rs`, `src/build_plan.rs`, `src/realization_routing.rs`, `src/portable_receipt.rs`, `src/source_bundle.rs`, `src/nickel_export.rs`, and stable release/attestation handoff DTOs selected by the inventory.
- **Consumers**: Cairn readiness, operator tooling, compatibility scripts, and sibling stack components get explicit version and failure contracts.
- **Compatibility**: existing JSON remains compatible unless it violates an already intended invariant; schema changes require explicit versioning and fixture migration.
- **Claims**: contract conformance proves public artifact shape and linkage invariants only, not build correctness, cache trust, reproducibility, release eligibility, attestation truth, or deployability.
