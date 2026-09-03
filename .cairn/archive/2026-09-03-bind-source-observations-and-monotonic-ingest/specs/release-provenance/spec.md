# Release Provenance Source Observation Delta

## ADDED Requirements

### Requirement: Release evidence binds accepted source observations

r[mantle.release_provenance.source_observation_binding] Mantle MUST bind an accepted source-observation identity and exact source content BLAKE3 into release evidence when the selected release profile requires source provenance.

#### Scenario: Matching source observation is bound

- **GIVEN** release source bytes match an accepted source observation and the selected profile requires source provenance
- **WHEN** Mantle creates and verifies release evidence
- **THEN** the release manifest MUST bind the observation schema, observation BLAKE3, source content BLAKE3, snapshot profile, and required non-claims
- **AND** the existing release attestation signature MUST cover the enclosing manifest identity

#### Scenario: Source observation is stale

- **GIVEN** the observation revision, projection, profile, content BLAKE3, observation BLAKE3, or release source bytes differ
- **WHEN** release verification evaluates the binding
- **THEN** verification MUST fail with a deterministic source-observation diagnostic
- **AND** stale source evidence MUST NOT satisfy reviewed-source, witness, or release policy

### Requirement: Source observation adds no signature authority

r[mantle.release_provenance.source_observation_signature_boundary] Mantle MUST reuse existing release, review, and witness signature roles and MUST NOT create a source-observation signature that can satisfy those roles.

#### Scenario: Observation has no independent signature

- **GIVEN** a valid source observation is included in a release-evidence manifest signed by an accepted release key
- **WHEN** release verification checks the source binding
- **THEN** the enclosing release signature MAY authenticate the manifest linkage
- **AND** the source observation MUST NOT count as source review, build witness, publisher ownership, or an additional signer

#### Scenario: Unknown source signature is supplied

- **GIVEN** an adapter supplies a signature over a source observation that is not part of an accepted external evidence profile
- **WHEN** Mantle evaluates release policy
- **THEN** it MUST ignore or reject that signature according to policy
- **AND** it MUST NOT promote the signature into release, review, witness, or ownership authority
