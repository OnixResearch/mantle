# Release Provenance Specification

## Purpose

Defines the `release-provenance` capability.

## Requirements

### Requirement: Valence stack provenance requirement policy
r[mantle.release_provenance.valence_required_policy] Mantle release profiles SHOULD be able to declare Valence stack-provenance sidecar evidence as optional or required, and required mode MUST fail closed when the sidecar or its Valence verification receipt is missing or invalid.

#### Scenario: Optional absent sidecar is recorded as absent
r[mantle.release_provenance.valence_required_policy.optional_absent]
- GIVEN a release profile where stack provenance is optional
- WHEN Mantle verifies a release bundle without a stack-provenance sidecar
- THEN verification MUST record an absent or skipped stack-provenance disposition without claiming the bundle carries stack provenance.

#### Scenario: Required sidecar passes with Valence receipt
r[mantle.release_provenance.valence_required_policy.required_valid]
- GIVEN a release profile where stack provenance is required and the bundle carries a matching sidecar plus Valence verification receipt
- WHEN Mantle verifies the release bundle
- THEN verification MUST accept the bundle-local stack-provenance evidence wiring.

#### Scenario: Required sidecar missing fails closed
r[mantle.release_provenance.valence_required_policy.required_missing]
- GIVEN a release profile where stack provenance is required
- WHEN the bundle has no stack-provenance sidecar or Valence verification receipt
- THEN verification MUST fail with deterministic diagnostics naming the missing evidence.

### Requirement: Valence verification receipt binding
r[mantle.release_provenance.valence_receipt_binding] Mantle release evidence MUST bind the stack-provenance sidecar artifact, Valence verification receipt artifact, BLAKE3 hashes, external evidence role, schema, claim scope, release binary identity, and required non-claims.

#### Scenario: Sidecar digest is verified
r[mantle.release_provenance.valence_receipt_binding.sidecar_digest]
- GIVEN a release bundle declares a stack-provenance sidecar artifact
- WHEN Mantle verifies the bundle
- THEN the sidecar bytes MUST match the BLAKE3 digest recorded in release evidence metadata.

#### Scenario: Valence receipt digest is verified
r[mantle.release_provenance.valence_receipt_binding.valence_receipt]
- GIVEN a release bundle declares a Valence stack-provenance verification receipt
- WHEN Mantle verifies the bundle
- THEN the receipt bytes MUST match the BLAKE3 digest recorded in release evidence metadata.

#### Scenario: Binary identity is linked
r[mantle.release_provenance.valence_receipt_binding.binary_identity]
- GIVEN stack-provenance evidence is present in a release bundle
- WHEN Mantle verifies the bundle
- THEN the release binary identity in bundle metadata MUST match the binary identity named by the sidecar or Valence receipt metadata available to Mantle.

#### Scenario: Stale evidence fails closed
r[mantle.release_provenance.valence_receipt_binding.stale]
- GIVEN the sidecar bytes, Valence receipt bytes, role, schema, claim scope, or binary identity do not match declared metadata
- WHEN Mantle verifies required stack-provenance evidence
- THEN verification MUST fail with deterministic diagnostics naming the stale or mismatched field.

### Requirement: Mantle opaque stack-provenance boundary
r[mantle.release_provenance.opaque_boundary] Mantle MUST treat stack provenance sidecars as opaque Valence-validated external evidence and MUST NOT claim to verify Octet, Trellis, Valence, or Cairn stack semantics itself.

#### Scenario: Opaque boundary is visible
r[mantle.release_provenance.opaque_boundary.visible]
- GIVEN release verification output or operator documentation describes stack-provenance sidecar handling
- WHEN the supported claim is stated
- THEN it MUST say Mantle validates bundle-local path, digest, role, schema, claim scope, binary identity, and non-claims while Valence owns stack semantics.

#### Scenario: Overclaiming boundary fails closed
r[mantle.release_provenance.opaque_boundary.overclaim]
- GIVEN release metadata claims Mantle semantically verified Octet, Trellis, Valence, or Cairn provenance semantics
- WHEN Mantle verifies the release bundle
- THEN verification MUST fail or downgrade the claim with a deterministic overclaim diagnostic.

### Requirement: Release provenance fixture matrix
r[mantle.release_provenance.fixture_matrix] Mantle MUST include positive and negative fixtures for optional and required Valence stack-provenance release evidence.

#### Scenario: Positive fixtures cover optional and required modes
r[mantle.release_provenance.fixture_matrix.positive]
- GIVEN optional-absent, optional-present, and required-present release evidence fixtures
- WHEN the fixture suite runs
- THEN optional absent MUST record an absent disposition, optional present MUST verify bundle-local metadata, and required present MUST pass with matching Valence receipt evidence.

#### Scenario: Negative fixtures cover required-mode failures
r[mantle.release_provenance.fixture_matrix.negative]
- GIVEN fixtures with missing sidecar, wrong role, wrong schema, wrong claim scope, stale sidecar digest, missing Valence receipt, stale Valence receipt digest, missing binary identity, or weakened non-claims
- WHEN the fixture suite runs in required mode
- THEN each fixture MUST fail closed with deterministic diagnostics.
