## ADDED Requirements

### Requirement: Admitted artifact export claims require current evidence [r[verification_evidence.admitted_artifact_export_proof_before_claim]]

Mantle MUST NOT claim that a frontend artifact can be exported, transferred, deployed, or used as a supported deploy handoff unless current evidence shows the artifact was spec-admitted and the export result preserved matching artifact identity, content digest, spec proof, and provenance.

#### Scenario: Export support claim cites receipt evidence [r[verification_evidence.admitted_artifact_export_proof_before_claim.scenario.claim]]

- GIVEN a task, evidence file, release note, status reply, or documentation page claims admitted-artifact export support
- WHEN the claim is made
- THEN the claim MUST cite current build-report, receipt, sidecar, or command evidence containing artifact ref, content digest, spec id, spec version, spec hash, validation result, and provenance
- AND the claim MUST be no broader than the inspected artifact export evidence.

#### Scenario: Missing export evidence blocks transfer claims [r[verification_evidence.admitted_artifact_export_proof_before_claim.scenario.missing]]

- GIVEN an artifact has a ref or spec-admission attestation but has not been exported through the admitted-artifact boundary
- WHEN status is reported or a Cairn task is considered complete
- THEN Mantle MUST NOT claim deploy transfer or artifact export support
- AND the report MUST state that admitted-artifact export evidence is missing.
