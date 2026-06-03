# Verification Evidence Specification

## Purpose

Defines Mantle's proof-before-claim evidence policy for status, completion, feature-support, and validation claims.

## Requirements

### Requirement: Proof before claim

r[verification_evidence.proof_before_claim] Mantle work MUST NOT claim status, completion, feature support, pass/fail validation, or build success unless current evidence has been produced or inspected before the claim is made.

#### Scenario: claim includes current evidence

- GIVEN an agent, task, evidence file, commit message, status reply, or final summary states that Mantle functionality works, validation passed, work is complete, or the tree is clean
- WHEN that claim is made
- THEN the claim MUST name or include current evidence such as command output, Cairn evidence file, task transcript, build report, test log, VCS status output, or inspected artifact content.
- AND the claim MUST be narrower than or equal to the evidence being cited.

#### Scenario: evidence is absent

- GIVEN current evidence has not been produced or inspected for a potential claim
- WHEN status is reported
- THEN the report MUST say the claim is not proven or state only the narrower fact that is supported by available evidence.
- AND it MUST NOT generalize from mock tests, stale logs, prior sessions, or uninspected artifacts to end-to-end support.

#### Scenario: Cairn task completion is evidence-gated

- GIVEN a Cairn task is marked complete
- WHEN the task records its completion summary
- THEN the task MUST reference durable evidence or an oracle checkpoint that supports the completed task text.
- AND review or gate summaries MUST reject completion claims whose evidence proves only a narrower behavior.

#### Scenario: post-archive validation claim is archived

- GIVEN a change has been archived
- WHEN a commit message, evidence summary, status reply, or final response claims post-archive Cairn validation passed
- THEN the archived change evidence transcript MUST include the exact post-archive validation command and output.
- AND the claim MUST NOT rely only on pre-archive gates or unstored chat transcript output.

### Requirement: Tracey coverage readiness

r[verification_evidence.tracey_coverage_readiness] Mantle MUST maintain deterministic coverage traceability for accepted Cairn requirements before presenting release-readiness or archive-readiness claims that depend on Tracey coverage.

#### Scenario: accepted requirements have traceability disposition

GIVEN an accepted requirement exists under `cairn/specs/`
WHEN Tracey coverage readiness is evaluated
THEN the requirement MUST have either implementation/verification references, an evidence-backed bridge reference, or an explicit tracked debt disposition.
AND comment-only bridge references MUST cite inspected implementation paths or durable evidence before being counted as satisfying traceability.

#### Scenario: coverage failures stay bounded

GIVEN `cairn tracey coverage --root . --json` reports missing requirements
WHEN an agent or operator reports readiness status
THEN the report MUST include the exact coverage validity, referenced count, missing count, dangling count, and next missing group.
AND it MUST NOT claim global Tracey coverage is green unless the command reports `valid: true`.

### Requirement: Spec admission claims require current evidence [r[verification_evidence.spec_admission_proof_before_claim]]

Mantle MUST NOT claim that a frontend artifact kind is spec-admitted, deployable, or supported unless current evidence shows the artifact manifest validated against the declared frontend spec and the build report or receipt contains the matching validation attestation.

#### Scenario: Supported frontend artifact claim cites attestation [r[verification_evidence.spec_admission_proof_before_claim.scenario.claim]]

GIVEN a task, evidence file, release note, status reply, or docs page claims support for a frontend artifact kind
WHEN the claim is made
THEN it MUST cite current build-report or receipt evidence containing spec id, version, hash, validator identity, artifact ref, validation result, and provenance
AND the claim MUST be no broader than the inspected spec-admitted artifact evidence.

#### Scenario: Missing attestation blocks support claim [r[verification_evidence.spec_admission_proof_before_claim.scenario.missing]]

GIVEN a build produced an artifact ref but no spec-validation attestation
WHEN status is reported or a Cairn task is considered complete
THEN Mantle MUST NOT claim the artifact kind is spec-admitted or deployable
AND the report MUST state that spec admission evidence is missing.
