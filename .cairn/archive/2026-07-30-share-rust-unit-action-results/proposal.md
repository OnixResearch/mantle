# Proposal: Share signed Rust unit action results

## Why

Local castore reuse does not help a fresh workstation or CI client. Mantle already separates immutable objects, action-result discovery, and execution through ADR 0024.

Rust unit results need the same separation. A remote index hit must remain advisory until Mantle verifies authority, identity, complete content, and the Rust-specific result contract.

## What Changes

- Add provider-neutral remote discovery for Rust unit result candidates by canonical action reference.
- Add signed Rust unit result envelopes with explicit producer and trust-policy identity.
- Transfer referenced castore content before final Rust unit admission.
- Publish immutable objects before signed records and publish index candidates last.
- Preserve conflicting admissible results as nondeterminism evidence.
- Add offline, ordered-source, redaction, failure, and byte-accounting behavior.
- Extend Rust topology receipts with local versus remote cache facts.

## Dependencies

This change depends on `persist-rust-unit-castore-results` and ADR 0024.

## Non-Goals

- Treating a remote index, server, object store, or signer name as output authority by itself.
- Replacing derivation `ActionResultRecord` or PathInfo semantics with Rust unit records.
- Remote execution.
- Compiler correctness or universal reproducibility claims.
- FUSE or virtiofs restoration into a writable target directory.

## Impact

- **Files**: `crates/crunch-rust-cache-core/`, `crates/crunch-rust-cache/`, action-result transport adapters, `src/rust_plan.rs`, CLI policy wiring, documentation, and Cairn evidence.
- **Testing**: clean-client hits, signed publication, authority failures, conflict handling, offline behavior, incomplete content, transport failures, and atomic visibility.
