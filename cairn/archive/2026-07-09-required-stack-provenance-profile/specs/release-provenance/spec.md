# Release Provenance Specification

## Purpose

Adds required stack-provenance release profiles for Onix stack artifacts.

## Requirements

### Requirement: Stack release profile requires provenance
r[mantle.release_provenance.stack_profile.required] Mantle MUST support a release profile for Onix stack artifacts that requires Valence stack-provenance sidecar and graph-report evidence while preserving optional stack provenance for generic releases.

#### Scenario: Profile-present evidence passes
r[mantle.release_provenance.stack_profile.positive]
- GIVEN a stack release profile requires stack provenance
- AND the bundle includes a Valence sidecar, Valence graph report, matching BLAKE3 digests, expected roles, expected schemas, supported claim scope, release-binary identity, and required non-claims
- WHEN `mantle release verify` evaluates the profile
- THEN verification MUST pass the stack-provenance contribution.

#### Scenario: Required profile fails absent evidence
r[mantle.release_provenance.stack_profile.required_absent]
- GIVEN a stack release profile requires stack provenance
- AND the bundle omits the sidecar or Valence graph report
- WHEN release verification runs
- THEN verification MUST fail closed with deterministic diagnostics naming the missing evidence.

### Requirement: Shared stack-provenance constants
r[mantle.release_provenance.stack_profile.constants] Mantle MUST derive or validate stack-provenance role, schema, claim-scope, and required non-claim constants from a reviewed source so CLI text, manifest validation, and documentation do not drift.

#### Scenario: Constants match profile contract
r[mantle.release_provenance.stack_profile.constants.match]
- GIVEN release verification uses stack-provenance role, schema, claim-scope, and non-claim constants
- WHEN the constants check runs
- THEN each constant MUST match the reviewed stack release profile contract.

#### Scenario: Constant drift fails
r[mantle.release_provenance.stack_profile.constants.drift]
- GIVEN a Rust constant, CLI example, or documentation string diverges from the reviewed profile contract
- WHEN the constants check runs
- THEN validation MUST fail with a drift diagnostic.

### Requirement: Required profile negative fixture matrix
r[mantle.release_provenance.stack_profile.negative] Mantle MUST include fail-closed fixtures for required stack-provenance profile failures.

#### Scenario: Invalid evidence is rejected
r[mantle.release_provenance.stack_profile.negative.invalid]
- GIVEN required stack-provenance evidence has a stale digest, wrong role, wrong schema, wrong binary link, unsupported claim scope, missing Valence receipt, or weakened non-claims
- WHEN release verification evaluates the stack profile
- THEN verification MUST fail closed with diagnostics identifying the invalid evidence class.

### Requirement: Profile documentation
r[mantle.release_provenance.stack_profile.docs] Mantle operator documentation MUST distinguish generic optional stack provenance from required Onix stack release profiles.

#### Scenario: Generic release remains optional
r[mantle.release_provenance.stack_profile.docs.generic]
- GIVEN an operator reads generic Mantle release docs
- WHEN stack provenance is described
- THEN the docs MUST state that generic releases default to optional stack provenance.

#### Scenario: Stack profile boundary is visible
r[mantle.release_provenance.stack_profile.docs.boundary]
- GIVEN an operator reads stack release profile docs
- WHEN required stack provenance is described
- THEN the docs MUST state that Mantle validates bundle-local evidence linkage only and Valence owns stack semantics.

### Requirement: Validation evidence
r[mantle.release_provenance.stack_profile.validation] The change MUST include focused release-evidence tests, profile/constant checks, and Cairn validation evidence before archive.

#### Scenario: Validation covers pass and fail cases
r[mantle.release_provenance.stack_profile.validation.fixtures]
- GIVEN positive and negative stack profile fixtures
- WHEN focused validation runs
- THEN valid fixtures MUST pass, invalid fixtures MUST fail closed, and saved evidence MUST bind profile and input identities.
