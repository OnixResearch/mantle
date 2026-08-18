## ADDED Requirements

### Requirement: Spec admission claims require current evidence [r[verification_evidence.spec_admission_proof_before_claim]]

Mantle MUST NOT claim that a frontend artifact kind is spec-admitted, deployable, or supported unless current evidence shows the artifact manifest validated against the declared frontend spec and the build report or receipt contains the matching validation attestation.

#### Scenario: Supported frontend artifact claim cites attestation [r[verification_evidence.spec_admission_proof_before_claim.scenario.claim]]

- GIVEN a task, evidence file, release note, status reply, or docs page claims support for a frontend artifact kind
- WHEN the claim is made
- THEN it MUST cite current build-report or receipt evidence containing spec id, version, hash, validator identity, artifact ref, validation result, and provenance
- AND the claim MUST be no broader than the inspected spec-admitted artifact evidence.

#### Scenario: Missing attestation blocks support claim [r[verification_evidence.spec_admission_proof_before_claim.scenario.missing]]

- GIVEN a build produced an artifact ref but no spec-validation attestation
- WHEN status is reported or a Cairn task is considered complete
- THEN Mantle MUST NOT claim the artifact kind is spec-admitted or deployable
- AND the report MUST state that spec admission evidence is missing.
