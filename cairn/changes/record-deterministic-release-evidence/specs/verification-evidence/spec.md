## ADDED Requirements

### Requirement: Release reproducibility evidence transcripts

r[verification_evidence.release_reproducibility_transcripts] Mantle MUST record durable, tracked evidence before claiming deterministic-release or release-reproducibility proof status for a specific release evidence bundle.

#### Scenario: transcript binds proof commands and digests

GIVEN a release evidence bundle is described as reproducible, deterministic-release eligible, or verified with a deterministic-release proof
WHEN the claim is recorded in a task, evidence file, release summary, or status reply
THEN a tracked evidence transcript MUST cite current command output for the reproduce command, required verifier command, and receipt checker command.
AND the transcript MUST record the release id, reproducibility report digest, deterministic proof digest, sandbox isolation evidence digest, rebuilt artifact digest set, and any provider fixed-point proof status used by the claim.

#### Scenario: generated proof payloads stay untracked

GIVEN deterministic release proof receipts, rebuild outputs, or release evidence bundles are produced under generated artifact directories
WHEN durable evidence is committed
THEN Mantle MUST commit concise lifecycle evidence text rather than generated proof payloads.
AND tracked evidence MUST identify the generated paths and digests without staging ignored `target/` artifacts.

#### Scenario: absent transcript blocks release proof claims

GIVEN a deterministic release proof was run only in an ephemeral working directory or chat transcript
WHEN Mantle reports release-readiness, archive-readiness, or completed release evidence status
THEN the report MUST say the deterministic-release claim is not durably recorded.
AND it MUST NOT present ephemeral generated artifacts as tracked release evidence until a lifecycle transcript exists.
