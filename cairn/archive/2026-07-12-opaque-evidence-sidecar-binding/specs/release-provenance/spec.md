# Release Provenance Specification

## Purpose

Bind generic opaque evidence-chain sidecars to Mantle release artifacts.

## Requirements

### Requirement: Mantle supports generic opaque evidence sidecars
r[mantle.release_provenance.opaque_evidence_sidecar_binding.contract] Mantle MUST support a generic release-evidence sidecar binding contract with canonical envelope hash, evidence kind, profile version, upstream validation hash, source artifact hash, release binary hash, policy hashes, claim scope, projections, and non-claims.

#### Scenario: Function-address uses generic sidecar binding
r[mantle.release_provenance.opaque_evidence_sidecar_binding.function_address]
- GIVEN function-address evidence is available as a canonical evidence-chain envelope and Valence validation receipt
- WHEN Mantle binds it to release artifacts
- THEN Mantle MUST use the generic opaque evidence sidecar binding path.

### Requirement: Release binding commits to typed links
r[mantle.release_provenance.opaque_evidence_sidecar_binding.links] Mantle MUST domain-separate sidecar identity, upstream validation identity, source artifact identity, release binary identity, and policy identity in release binding receipts.

#### Scenario: Source and binary linkage is checked
r[mantle.release_provenance.opaque_evidence_sidecar_binding.positive]
- GIVEN a valid sidecar binding references matching source and release binary artifacts
- WHEN release validation runs
- THEN Mantle MUST emit a deterministic binding receipt that commits to those typed links.

### Requirement: Evidence payloads remain opaque to Mantle core
r[mantle.release_provenance.opaque_evidence_sidecar_binding.opaque_core] Mantle release validation core MUST NOT parse profile payloads, Rust source, proof terms, or Preserves internals for opaque evidence sidecars.

#### Scenario: Core validates metadata only
r[mantle.release_provenance.opaque_evidence_sidecar_binding.validation]
- GIVEN the CLI has loaded and hashed sidecar metadata
- WHEN release validation core runs
- THEN the core MUST evaluate typed in-memory metadata and deterministic diagnostics without filesystem or parser effects.

### Requirement: Invalid sidecar bindings fail closed
r[mantle.release_provenance.opaque_evidence_sidecar_binding.negative] Mantle MUST reject or mark invalid opaque sidecar bindings with unknown evidence kind, missing canonical hash, stale upstream validation hash, source/binary mismatch, unsupported claim scope, wrong role/schema, projection drift, malformed BLAKE3, or weakened non-claims.

#### Scenario: Non-function-address stub proves generality
- GIVEN a supported non-function-address evidence kind has a canonical envelope and upstream validation receipt
- WHEN Mantle validates release binding metadata
- THEN Mantle MUST bind it through the same generic sidecar path without adding profile-specific release code.
