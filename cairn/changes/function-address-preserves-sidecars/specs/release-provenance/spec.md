# Release Provenance Specification

## Purpose

Bind canonical Kamacite Preserves function-address receipts as opaque release evidence sidecars.

## Requirements

### Requirement: Mantle binds Preserves receipt sidecar metadata
r[mantle.release_provenance.function_address_preserves_sidecars.contract] Mantle MUST support function-address release evidence metadata for canonical Kamacite Preserves receipt sidecars, including role, schema version, canonical BLAKE3 receipt hash, source archive digest, release binary digest, Valence receipt hash, claim scope, and non-claims.

#### Scenario: Required Preserves-backed binding passes
r[mantle.release_provenance.function_address_preserves_sidecars.positive]
- GIVEN release evidence includes matching source archive, release binary, Valence receipt, and canonical Kamacite Preserves receipt metadata
- WHEN Mantle validates function-address release evidence in required mode
- THEN Mantle MUST emit a passing release binding receipt.

### Requirement: Mantle treats Preserves receipts as opaque evidence
r[mantle.release_provenance.function_address_preserves_sidecars.opaque] Mantle MUST NOT parse Rust functions or reinterpret Preserves receipt internals when binding function-address release evidence.

#### Scenario: Core validates typed metadata only
r[mantle.release_provenance.function_address_preserves_sidecars.validation]
- GIVEN the CLI has loaded and hashed sidecar artifacts
- WHEN release validation core runs
- THEN the core MUST evaluate typed metadata and deterministic diagnostics without filesystem reads or Preserves parser effects.

### Requirement: Compatibility projections are bound to canonical identity
r[mantle.release_provenance.function_address_preserves_sidecars.json_projection] Mantle MAY accept JSON compatibility sidecars only when they bind to the canonical Kamacite Preserves receipt identity.

#### Scenario: Invalid sidecar binding fails closed
r[mantle.release_provenance.function_address_preserves_sidecars.negative]
- GIVEN function-address release evidence has missing or stale Preserves hash, stale Valence hash, projection drift, wrong role/schema, unsupported claim scope, malformed BLAKE3, source/binary mismatch, or overclaiming text
- WHEN Mantle validates the binding
- THEN validation MUST fail with deterministic release-evidence diagnostics.
