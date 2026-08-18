# Release Provenance Specification

## Purpose

Bind Trellis proof evidence-chain sidecars to Mantle release artifacts.

## ADDED Requirements

### Requirement: Mantle supports Trellis proof evidence sidecars
r[mantle.release_provenance.trellis_proof_sidecars.profile] Mantle MUST support Trellis proof evidence as a profile of the generic opaque evidence sidecar binding contract.

#### Scenario: Accepted proof sidecar binds to release
r[mantle.release_provenance.trellis_proof_sidecars.positive]
- GIVEN release evidence includes a canonical Kamacite Trellis proof envelope and Valence validation receipt for accepted proof evidence
- WHEN Mantle validates release provenance in required mode
- THEN Mantle MUST emit a passing binding receipt if source and binary links match.

### Requirement: Trellis proof binding commits to typed links
r[mantle.release_provenance.trellis_proof_sidecars.links] Mantle MUST bind canonical envelope hash, Valence validation hash, source archive hash, release binary hash, policy hashes, proof role, claim scope, and non-claims for Trellis proof sidecars.

#### Scenario: Stale upstream validation fails
r[mantle.release_provenance.trellis_proof_sidecars.negative]
- GIVEN a Trellis proof sidecar binding has a stale Valence validation hash, missing canonical envelope hash, source/binary mismatch, unsupported profile version, wrong role, unsupported claim scope, projection drift, malformed BLAKE3, or overclaiming text
- WHEN release validation runs
- THEN validation MUST fail with deterministic release-evidence diagnostics.

### Requirement: Proof payloads stay opaque to Mantle core
r[mantle.release_provenance.trellis_proof_sidecars.opaque] Mantle release validation core MUST NOT parse Verus source, proof IR, verifier logs, or Preserves internals for Trellis proof evidence sidecars.

#### Scenario: Recorded-only proof evidence remains bounded
r[mantle.release_provenance.trellis_proof_sidecars.validation]
- GIVEN a release carries recorded-only Trellis proof evidence
- WHEN Mantle binds the sidecar
- THEN the receipt MUST preserve recorded-only role metadata and MUST NOT claim proof acceptance, Rust semantic correctness, or release eligibility.
