# Release Provenance Specification

## Purpose

Adds generic Cairn evidence handoff roles for Mantle release bundles.

## Requirements

### Requirement: Cairn evidence handoff contract
r[mantle.release_provenance.cairn_evidence_handoff.contract] Mantle MUST support release external-evidence roles for Cairn evidence exports, verification receipts, indexes, and release-readiness receipts as opaque evidence.

#### Scenario: Complete Cairn evidence handoff passes
r[mantle.release_provenance.cairn_evidence_handoff.fixtures.positive]
- GIVEN a release bundle includes Cairn evidence artifacts with expected roles, schemas, digests, claim scope, and non-claims
- WHEN release verification runs
- THEN Mantle MUST accept the handoff as bundle-local evidence linkage.

#### Scenario: Invalid Cairn evidence handoff fails
r[mantle.release_provenance.cairn_evidence_handoff.fixtures.negative]
- GIVEN a Cairn evidence handoff is missing, stale, wrong-role, wrong-schema, or missing non-claims
- WHEN release verification runs
- THEN Mantle MUST fail closed with deterministic diagnostics.

### Requirement: Handoff validation boundary
r[mantle.release_provenance.cairn_evidence_handoff.validation] Mantle MUST validate Cairn evidence handoff path, digest, role, schema, claim scope, and non-claims only.

#### Scenario: Boundary is visible
r[mantle.release_provenance.cairn_evidence_handoff.docs]
- GIVEN a Cairn handoff validates
- WHEN the supported claim is rendered
- THEN it MUST state that Cairn owns lifecycle semantics and Mantle validates bundle-local linkage only.

### Requirement: Final validation
r[mantle.release_provenance.cairn_evidence_handoff.final_validation] The change MUST include positive and negative fixtures plus focused validation evidence before archive.

#### Scenario: Fixture suite covers handoff health
r[mantle.release_provenance.cairn_evidence_handoff.final_validation.fixtures]
- GIVEN valid and invalid Cairn handoff fixtures
- WHEN focused validation runs
- THEN valid fixtures MUST pass and invalid fixtures MUST fail closed.
