# Release Provenance Specification

## Purpose

Bind canonical Kamacite Preserves function-address receipts as opaque release evidence sidecars.

## Requirements

### Requirement: Mantle binds Preserves receipt sidecar metadata
r[mantle.release_provenance.function_address_preserves_sidecars.contract] Mantle MUST support function-address release evidence metadata for canonical Kamacite Preserves receipt sidecars, including role, schema version, canonical BLAKE3 receipt hash, bounded byte size, source archive digest, release binary digest, Valence artifact and logical receipt hashes, claim scope, and non-claims.

#### Scenario: Required Preserves-backed binding passes
r[mantle.release_provenance.function_address_preserves_sidecars.positive]
- GIVEN release evidence includes matching source archive, release binary, Valence receipt, and canonical Kamacite Preserves receipt metadata
- WHEN Mantle validates function-address release evidence in required mode
- THEN Mantle MUST emit a passing release binding receipt.

#### Scenario: Manifest-driven CLI rejects mixed-time identity drift
- GIVEN a verified release manifest contains exactly one typed `function-address-preserves-v1` binding
- WHEN `mantle release function-address-bind --from-preserves-binding` reopens its sidecars
- THEN Mantle MUST perform bounded no-follow reads, compare reopened byte digests and canonical Preserves size with the manifest, and fail before output on drift.

### Requirement: Mantle treats Preserves receipts as opaque evidence
r[mantle.release_provenance.function_address_preserves_sidecars.opaque] Mantle MUST NOT parse Rust functions or reinterpret Preserves receipt internals when binding function-address release evidence.

#### Scenario: Core validates typed metadata only
r[mantle.release_provenance.function_address_preserves_sidecars.validation]
- GIVEN the CLI has loaded and hashed sidecar artifacts
- WHEN release validation core runs
- THEN the core MUST evaluate typed metadata and deterministic diagnostics without filesystem reads or Preserves parser effects.

### Requirement: Compatibility projections are bound to canonical identity
r[mantle.release_provenance.function_address_preserves_sidecars.json_projection] Mantle MAY accept JSON compatibility sidecars only when their distinct artifact-byte digest and public logical `receipt_hash` bind to the canonical Kamacite Preserves receipt identity.

#### Scenario: Invalid sidecar binding fails closed
r[mantle.release_provenance.function_address_preserves_sidecars.negative]
- GIVEN function-address release evidence has missing or stale Preserves hash, stale Valence hash, projection drift, wrong role/schema, unsupported claim scope, malformed BLAKE3, source/binary mismatch, or overclaiming text
- WHEN Mantle validates the binding
- THEN validation MUST fail with deterministic release-evidence diagnostics.
