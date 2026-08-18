## ADDED Requirements

### Requirement: Refreshed provider-bound release evidence transcripts

r[verification_evidence.provider_bound_release_evidence_refresh_transcripts] Mantle MUST record durable refreshed evidence before claiming a newer current-code provider fixed-point proof is packaged as provider-bound deterministic release evidence.

#### Scenario: refreshed transcript binds current provider proof to a release artifact

GIVEN a prior provider-bound release evidence transcript recorded a current-code provider proof blocker
WHEN a newer provider fixed-point proof succeeds and is packaged into a release evidence bundle
THEN a tracked evidence transcript MUST cite current command output showing the provider proof status is valid and the provider proof matched a release artifact relative path and BLAKE3 digest.
AND the transcript MUST record the release id, matched provider artifact path, matched provider artifact digest, provider proof digest, deterministic proof digest, sandbox isolation evidence digest, and verifier status.

#### Scenario: refreshed portable replay proves copied artifact sufficiency

GIVEN refreshed provider-bound release evidence is intended to supersede a prior blocker note
WHEN portable replay evidence is recorded
THEN the transcript MUST cite current positive verification from copied artifacts with deterministic-release and provider fixed-point gates enabled.
AND it MUST cite a negative replay showing missing required proof material fails closed instead of reporting eligibility.

#### Scenario: refreshed evidence stays bounded

GIVEN refreshed provider-bound deterministic release verification succeeds
WHEN the evidence is summarized
THEN the summary MUST state that the claim is limited to the recorded packaged artifact set and proof inputs.
AND it MUST NOT claim full bootstrap reproducibility, compiler correctness, deploy success, or full Cargo compatibility unless separate evidence proves those claims.
