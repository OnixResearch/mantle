# Release Provenance Specification

## Purpose

Requires production Cairn handoff validation over measured bytes and blocks release paths that bypass the validator.

## Requirements

### Requirement: Cairn handoff inputs bind measured bytes
r[mantle.release_provenance.cairn_evidence_handoff.measured_inputs] Mantle MUST measure explicitly supplied Cairn handoff artifact bytes in the shell and pass typed measured identities to the pure handoff validator.

#### Scenario: Matching handoff passes
r[mantle.release_provenance.cairn_evidence_handoff.fixtures.positive]
- GIVEN a handoff row's artifact id, role, schema, measured digest, policy digest, readiness id, coverage ids, and non-claim boundary match the loaded artifacts
- WHEN production handoff validation runs
- THEN the row MAY pass as bundle-local external evidence.

#### Scenario: Tampered or fabricated identity fails
r[mantle.release_provenance.cairn_evidence_handoff.fixtures.negative]
- GIVEN a handoff row contains a digest-shaped value that does not match the referenced bytes
- WHEN validation compares declared and measured identity
- THEN validation MUST fail before the row contributes to release evidence.

### Requirement: Production release paths invoke Cairn handoff validation
r[mantle.release_provenance.cairn_evidence_handoff.production_wiring] Mantle release assembly and release verification MUST invoke the pure Cairn handoff validator for every present handoff row before emitting a passing bundle or release receipt.

#### Scenario: Release verification validates present handoff
- GIVEN `mantle release verify` loads a bundle containing Cairn handoff evidence
- WHEN release verification runs
- THEN it MUST validate the loaded handoff and bind the validation result into the release verification receipt.

#### Scenario: Test-only reachability is insufficient
- GIVEN the handoff validator is covered by unit tests but no production release path invokes it
- WHEN release-readiness validation evaluates implementation coverage
- THEN the change MUST remain incomplete.

### Requirement: Required profiles block validator bypass
r[mantle.release_provenance.cairn_evidence_handoff.bypass_protection] A release profile requiring Cairn handoff evidence MUST fail when validation was not invoked, its receipt is absent, or its receipt is bound to another release bundle.

#### Scenario: Parsed metadata cannot bypass validation
- GIVEN handoff metadata parses successfully but no matching validation receipt exists
- WHEN a required release profile is evaluated
- THEN release verification MUST fail with a handoff-validation-missing diagnostic.

### Requirement: Authenticated Cairn dependency is explicit
r[mantle.release_provenance.cairn_evidence_handoff.cross_repo_dependency] Mantle MUST require the archived Cairn authenticated-input change receipt before accepting a Cairn handoff as authenticated release evidence.

#### Scenario: Legacy unauthenticated handoff remains bounded
- GIVEN Cairn handoff evidence predates authenticated measured-byte and producer-policy checks
- WHEN Mantle evaluates it
- THEN Mantle MUST reject it for an authenticated required profile or classify it as non-release legacy context.

### Requirement: Cairn handoff boundary is documented
r[mantle.release_provenance.cairn_evidence_handoff.docs] Mantle documentation and receipts MUST state that production handoff validation proves bundle-local measured identity and linkage only.

#### Scenario: Stronger claims remain excluded
- GIVEN a Cairn handoff validates
- WHEN Mantle renders the result
- THEN it MUST NOT claim Cairn correctness, source correctness, build correctness, semantic equivalence, deployment safety, or universal release fitness.

### Requirement: Production handoff verification rail
r[mantle.release_provenance.cairn_evidence_handoff.final_validation] The change MUST include positive and negative evidence that exercises real production call paths, measured-byte comparison, bypass protection, and release-bundle binding.

#### Scenario: Production fixture detects bypass and tampering
- GIVEN valid, bypassed, stale, and tampered release fixtures
- WHEN focused validation runs
- THEN valid production wiring MUST pass and every bypassed or mismatched fixture MUST fail closed.

### Requirement: Flake-check CI is checked in
r[mantle.release_provenance.cairn_evidence_handoff.flake_check_ci] Mantle MUST include a checked-in CI workflow for this remediation whose verification command is `nix flake check`.

#### Scenario: CI uses the scoped verification rail
- GIVEN a change is evaluated by checked-in CI
- WHEN the remediation workflow runs
- THEN it MUST execute `nix flake check` without requiring a separate expanded CI command matrix in this change.
