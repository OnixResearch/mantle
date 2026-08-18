## ADDED Requirements

### Requirement: Provider-bound release evidence transcripts

r[verification_evidence.provider_bound_release_evidence_transcripts] Mantle MUST record durable evidence before claiming provider-bound deterministic release evidence for a specific release evidence bundle.

#### Scenario: transcript binds provider proof to a release artifact

GIVEN a release evidence bundle is described as provider-bound, deterministic-release eligible, or verified with provider fixed-point proof evidence
WHEN the claim is recorded in a task, evidence file, release summary, or status reply
THEN a tracked evidence transcript MUST cite current command output showing the provider proof status is valid and the provider proof matched a release artifact relative path and BLAKE3 digest.
AND the transcript MUST record the release id, matched provider artifact path, matched provider artifact digest, provider proof digest, deterministic proof digest, sandbox isolation evidence digest, and verifier status.

#### Scenario: portable replay proves copied artifact sufficiency

GIVEN provider-bound release evidence is intended to be replayable outside the original bundle path
WHEN portable replay evidence is recorded
THEN the transcript MUST cite current positive verification from copied artifacts with deterministic-release and provider fixed-point gates enabled.
AND it MUST cite a negative replay showing missing required proof material fails closed instead of reporting eligibility.

#### Scenario: provider-bound release evidence stays bounded

GIVEN provider-bound deterministic release verification succeeds
WHEN the evidence is summarized
THEN the summary MUST state that the claim is limited to the recorded packaged artifact set and proof inputs.
AND it MUST NOT claim full bootstrap reproducibility, compiler correctness, deploy success, or full Cargo compatibility unless separate evidence proves those claims.
