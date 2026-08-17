# Release Provenance Specification Delta

## ADDED Requirements

### Requirement: Release coverage binds content-bound requirements

r[mantle.release_provenance.content_bound_requirement_coverage]

Mantle MUST support versioned release coverage that binds each requirement to exact Cairn registry and Valence reference identities.

#### Scenario: Current typed coverage passes

- GIVEN a release coverage row matches the selected Cairn registry and Valence requirement reference
- WHEN the pure release core validates the row
- THEN validation MUST preserve repository, revision, specification, policy, registry, and requirement-row identities
- AND the result MUST remain bounded identity and linkage evidence

#### Scenario: Stale or wrong-repository coverage fails

- GIVEN a coverage row has a stale registry identity or names another repository's same-named requirement
- WHEN strict release verification evaluates the row
- THEN verification MUST fail with a deterministic issue
- AND the release MUST NOT report the requirement as validly covered

#### Scenario: Duplicate coverage fails

- GIVEN two release rows claim the same typed requirement and evidence role
- WHEN the release core canonicalizes coverage
- THEN validation MUST fail before release-manifest hashing
- AND input order MUST NOT hide the duplication

### Requirement: Strict release coverage binds exact evidence artifacts

r[mantle.release_provenance.content_bound_evidence_manifest]

Mantle MUST bind strict requirement coverage to a machine-readable evidence manifest containing exact content identities and typed evidence roles.

#### Scenario: Exact source and test evidence passes

- GIVEN manifest rows name safe paths, current BLAKE3 content, bounded locations, roles, and producer receipt refs
- WHEN release creation measures and validates the supplied evidence
- THEN the release MUST bind each accepted row into the evidence-manifest identity
- AND release verification MUST recheck every required relationship

#### Scenario: Bridge-only linkage is insufficient

- GIVEN a requirement appears only in the human-readable traceability bridge
- WHEN strict release policy requires content-bound evidence rows
- THEN release verification MUST fail with a missing-evidence-row issue
- AND the bridge comment MUST NOT be treated as current file-content proof

#### Scenario: Present optional evidence is stale

- GIVEN an optional evidence role is not required but a stale row is supplied
- WHEN strict verification evaluates the release
- THEN verification MUST fail rather than ignore the invalid row
- AND absent and invalid-present evidence MUST remain distinct

### Requirement: Legacy release coverage preserves a lower-fidelity boundary

r[mantle.release_provenance.legacy_coverage_boundary]

Mantle MUST keep existing string coverage and traceability bridges readable without promoting them into strict content-bound evidence.

#### Scenario: Existing release remains readable

- GIVEN an existing release contains only string coverage vectors
- WHEN the compatibility profile validates it
- THEN Mantle MUST preserve its original schema and identity
- AND it MUST retain explicit non-claims for registry membership and evidence freshness

#### Scenario: Strict policy rejects legacy-only coverage

- GIVEN strict policy requires content-bound requirement and evidence rows
- WHEN a release supplies only legacy vectors or bridge comments
- THEN verification MUST fail with a migration diagnostic
- AND it MUST NOT claim that the underlying code or requirement is incorrect

#### Scenario: Selected-revision integration remains bounded

- GIVEN a cross-repository fixture passes for exact Cairn and Valence revisions
- WHEN Mantle records the integration receipt
- THEN the receipt MUST bind both revisions and schema identities
- AND it MUST NOT claim compatibility with arbitrary revisions
