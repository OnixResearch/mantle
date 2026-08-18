# Release Provenance Specification

## Purpose

Adds a reusable Nix evidence core for Mantle release provenance.

## Requirements

### Requirement: Nix evidence core contract
r[mantle.release_provenance.nix_evidence_core.contract] Mantle MUST define pure Nix evidence contracts for store path refs, derivation/output identities, realization roles, artifact digest metadata, build caveats, and bounded non-claims.

#### Scenario: Valid Nix evidence row validates
r[mantle.release_provenance.nix_evidence_core.fixtures.positive]
- GIVEN a Nix evidence row has a supported store path ref, derivation/output identity, realization role, artifact digest metadata, build caveats, and non-claim boundary
- WHEN Nix evidence validation runs
- THEN validation MUST accept the row and preserve the role-specific identity.

#### Scenario: Invalid Nix evidence row fails closed
r[mantle.release_provenance.nix_evidence_core.fixtures.negative]
- GIVEN a Nix evidence row has a malformed store path, wrong output name, digest mismatch, unsupported derivation identity, missing caveat, ambiguous role, missing non-claim, or build-correctness overclaim
- WHEN Nix evidence validation runs
- THEN validation MUST fail with deterministic diagnostics naming the invalid evidence field.

### Requirement: Nix evidence roles remain distinct
r[mantle.release_provenance.nix_evidence_core.roles] Mantle MUST preserve role separation between build outputs, fixed-output fetches, release bundle members, sidecars, and external evidence rows.

#### Scenario: Same output digest cannot erase role
r[mantle.release_provenance.nix_evidence_core.roles.same_digest]
- GIVEN two Nix evidence rows share an output digest but have different realization roles
- WHEN validation compares the rows
- THEN validation MUST preserve the role distinction and MUST NOT silently substitute one role for another.

### Requirement: Nix evidence validation is pure
r[mantle.release_provenance.nix_evidence_core.validation] Mantle MUST implement Nix evidence validation as pure deterministic logic over in-memory store refs, derivation/output rows, role labels, digest metadata, caveats, and non-claims.

#### Scenario: Shell owns build and store effects
r[mantle.release_provenance.nix_evidence_core.validation.shell]
- GIVEN a CLI evaluates derivations, reads store paths, contacts substituters, runs builds, or captures sandbox evidence
- WHEN Nix evidence validation is invoked
- THEN those effects MUST remain outside the Nix evidence core.

### Requirement: Compatibility adapters preserve public reports
r[mantle.release_provenance.nix_evidence_core.adapters] Mantle SHOULD provide adapters for current build reports, release provenance rows, Cairn Nix gate evidence, Molten release promotion evidence, and Valence provenance inputs.

#### Scenario: Build report fields remain public-compatible
r[mantle.release_provenance.nix_evidence_core.adapters.build_report]
- GIVEN a current Mantle build report exposes public Nix evidence fields
- WHEN an adapter validates the report
- THEN the adapter SHOULD preserve those public fields while routing reusable Nix evidence checks through the shared core.

### Requirement: Nix evidence core remains bounded
r[mantle.release_provenance.nix_evidence_core.docs] Passing Nix evidence validation MUST NOT prove build correctness, source-code correctness, semantic equivalence, sandbox soundness, release eligibility, lifecycle readiness, or verifier soundness.

#### Scenario: Boundary is visible
r[mantle.release_provenance.nix_evidence_core.final_validation]
- GIVEN a Nix evidence validation report passes
- WHEN an operator reads the supported claim
- THEN the report MUST state that validation proves only store-shaped identity, role separation, digest metadata consistency, caveat presence, and non-claim conformance.
