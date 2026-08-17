# Release Provenance Specification

## Purpose

Adds Preserves release evidence carrier roles for Mantle bundles.

## Requirements

### Requirement: Preserves carrier contract
r[mantle.release_provenance.preserves_carriers.contract] Mantle MUST support release external-evidence rows for Preserves carriers with bundle path, schema label, producer repo, canonical digest, evidence role, claim scope, opacity mode, optional adapter ID, and non-claims.

#### Scenario: Canonical Preserves carrier passes
r[mantle.release_provenance.preserves_carriers.fixtures.positive]
- GIVEN a release bundle includes a Preserves carrier with canonical bytes, supported schema label, BLAKE3 identity, producer repo, role, claim scope, and required non-claims
- WHEN release evidence validation runs
- THEN Mantle MUST accept the carrier as bundle-local evidence linkage.

#### Scenario: Invalid Preserves carrier fails
r[mantle.release_provenance.preserves_carriers.fixtures.negative]
- GIVEN a carrier has stale digest, wrong schema, wrong role, missing non-claims, unsupported adapter, or semantic overclaim
- WHEN release evidence validation runs
- THEN Mantle MUST fail closed with deterministic diagnostics.

### Requirement: Carrier validation boundary
r[mantle.release_provenance.preserves_carriers.validation] Mantle MUST validate Preserves carrier identity, role, schema, claim scope, and non-claims only unless a release profile declares a bounded adapter.

#### Scenario: Opaque carrier stays opaque
r[mantle.release_provenance.preserves_carriers.validation.opaque]
- GIVEN an opaque Preserves carrier validates
- WHEN the supported claim is rendered
- THEN it MUST state that Mantle does not prove payload semantics, producer runtime behavior, authorization correctness, or Cairn/Valence acceptance.

### Requirement: Carrier docs
r[mantle.release_provenance.preserves_carriers.docs] Documentation MUST describe Preserves carriers as release evidence containers, not release eligibility proof by themselves.

#### Scenario: Non-claim is visible
r[mantle.release_provenance.preserves_carriers.docs.non_claims]
- GIVEN a Preserves carrier is present in a release bundle
- WHEN release evidence is summarized
- THEN the summary MUST show the bounded carrier claim and required linked receipts for stronger claims.

### Requirement: Final validation
r[mantle.release_provenance.preserves_carriers.final_validation] The change MUST include positive and negative fixtures plus focused validation evidence before archive.

#### Scenario: Fixture suite covers carrier health
r[mantle.release_provenance.preserves_carriers.final_validation.fixtures]
- GIVEN valid and invalid Preserves carrier fixtures
- WHEN focused validation runs
- THEN valid fixtures MUST pass and invalid fixtures MUST fail closed.
