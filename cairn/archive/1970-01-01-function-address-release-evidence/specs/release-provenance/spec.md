# Release Provenance Specification

## Purpose

Extends Mantle release provenance with bundle-local binding for Valence/Kamacite function-address evidence while keeping function-address semantics outside Mantle.

## Requirements

### Requirement: Function-address evidence policy

r[mantle.release_provenance.function_address_evidence.policy] Mantle release profiles SHOULD support disabled, optional, and required modes for function-address evidence. Required mode MUST fail closed when the function-address sidecar, Valence verification receipt or graph report, expected roles, expected schemas, claim scope, binary/source identity, BLAKE3 digests, or required non-claims are missing or invalid.

#### Scenario: Optional absent evidence is recorded
r[mantle.release_provenance.function_address_evidence.policy.optional_absent]
- GIVEN a release profile where function-address evidence is optional
- WHEN Mantle verifies a release bundle without function-address evidence
- THEN verification MUST record an absent or skipped disposition without claiming the bundle carries function-address evidence.

#### Scenario: Required evidence passes with matching sidecars
r[mantle.release_provenance.function_address_evidence.policy.required_valid]
- GIVEN a release profile where function-address evidence is required and the bundle carries matching sidecar, Valence receipt, binary identity, source identity, roles, schemas, and non-claims
- WHEN Mantle verifies the release bundle
- THEN verification MUST accept the function-address evidence contribution as bundle-local linkage.

#### Scenario: Required evidence missing fails
r[mantle.release_provenance.function_address_evidence.policy.required_missing]
- GIVEN a release profile where function-address evidence is required
- WHEN the bundle omits the function-address sidecar or Valence verification receipt
- THEN release verification MUST fail with deterministic diagnostics naming the missing evidence.

### Requirement: Function-address evidence binding

r[mantle.release_provenance.function_address_evidence.binding] Mantle MUST bind function-address evidence to release metadata by checking sidecar byte digest, Valence receipt or graph-report byte digest, Kamacite receipt identity when present, release binary identity, source artifact identity, external evidence role, schema, claim scope, and required non-claims.

#### Scenario: Sidecar and receipt digests are verified
r[mantle.release_provenance.function_address_evidence.binding.digests]
- GIVEN a release bundle declares function-address sidecar and Valence receipt artifacts
- WHEN Mantle verifies the bundle
- THEN the sidecar and receipt bytes MUST match the BLAKE3 digests recorded in release evidence metadata.

#### Scenario: Binary and source identities are linked
r[mantle.release_provenance.function_address_evidence.binding.binary_source]
- GIVEN function-address evidence is present in a release bundle
- WHEN Mantle verifies the bundle
- THEN the release binary identity and source artifact identity in bundle metadata MUST match the identities named by the function-address evidence metadata available to Mantle.

#### Scenario: Stale binding fails
r[mantle.release_provenance.function_address_evidence.binding.stale]
- GIVEN the sidecar bytes, Valence receipt bytes, role, schema, claim scope, binary identity, source identity, or non-claims do not match declared metadata
- WHEN Mantle verifies required function-address evidence
- THEN verification MUST fail with deterministic diagnostics naming the stale or mismatched field.

### Requirement: Function-address validation boundary

r[mantle.release_provenance.function_address_evidence.validation] Mantle MUST validate function-address release evidence as pure validation over loaded metadata rows while file reads, digest measurement, bundle assembly, and upstream evidence production remain outside the pure core.

#### Scenario: Mantle does not parse functions
r[mantle.release_provenance.function_address_evidence.validation.opaque]
- GIVEN a function-address sidecar contains individual function records
- WHEN Mantle verifies release evidence
- THEN Mantle MUST treat the sidecar as opaque external evidence and MUST NOT recompute or interpret individual function addresses.

#### Scenario: Boundary is visible in output
r[mantle.release_provenance.function_address_evidence.validation.boundary]
- GIVEN function-address release evidence passes
- WHEN Mantle reports the result
- THEN the supported claim MUST be limited to bundle-local path, digest, role, schema, claim-scope, binary/source identity, and non-claim validation.

### Requirement: Positive fixture coverage

r[mantle.release_provenance.function_address_evidence.fixtures.positive] Mantle MUST include positive fixtures for optional-absent, optional-present, and required-present function-address release evidence.

#### Scenario: Positive fixtures cover modes
r[mantle.release_provenance.function_address_evidence.fixtures.positive.modes]
- GIVEN optional absent, optional present, and required present release evidence fixtures
- WHEN the fixture suite runs
- THEN optional absent MUST record an absent disposition, optional present MUST verify bundle-local metadata, and required present MUST pass with matching Valence/Kamacite evidence metadata.

### Requirement: Negative fixture coverage

r[mantle.release_provenance.function_address_evidence.fixtures.negative] Mantle MUST include fail-closed fixtures for missing sidecars, missing Valence receipts, stale BLAKE3 digests, wrong roles, wrong schemas, unsupported claim scopes, binary identity mismatch, source identity mismatch, weakened non-claims, and overclaims.

#### Scenario: Negative fixtures fail required mode
r[mantle.release_provenance.function_address_evidence.fixtures.negative.required]
- GIVEN invalid required-mode function-address release evidence fixtures
- WHEN release verification evaluates the fixtures
- THEN each fixture MUST fail closed with deterministic diagnostics naming the invalid evidence class.

### Requirement: Function-address evidence documentation

r[mantle.release_provenance.function_address_evidence.docs] Mantle operator documentation MUST distinguish optional generic release evidence from required Onix stack release evidence and MUST preserve the opaque external-evidence boundary.

#### Scenario: Docs preserve ownership boundary
r[mantle.release_provenance.function_address_evidence.docs.boundary]
- GIVEN an operator reads function-address release evidence documentation
- WHEN the evidence flow is described
- THEN the docs MUST state that Octet owns Rust extraction, Kamacite owns portable receipts, Valence owns linkage semantics, Mantle owns bundle-local binding, and Cairn owns lifecycle/readiness policy.

### Requirement: Final validation evidence

r[mantle.release_provenance.function_address_evidence.final_validation] The change MUST include positive and negative release fixtures, focused release-provenance tests, constants/profile checks, Cairn validation, and proposal/design/tasks gates before archive.

#### Scenario: Validation covers pass and fail cases
r[mantle.release_provenance.function_address_evidence.final_validation.fixtures]
- GIVEN valid and invalid function-address release evidence fixtures
- WHEN focused validation runs
- THEN valid fixtures MUST pass, invalid fixtures MUST fail closed, and saved evidence MUST bind profile and input identities.
