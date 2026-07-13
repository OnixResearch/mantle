# Verification Evidence Specification

## Purpose

Define evidence required before Mantle claims production resumable remote transfer is complete.

## Requirements

### Requirement: Production transfer completion claims require interruption evidence [r[verification_evidence.production_transfer_completion_claim]]

Mantle MUST NOT claim production streaming or resumable remote transfer complete while the production path still requires whole inline payloads or lacks current positive and negative interruption evidence. Completion evidence MUST identify the production code path, transfer manifest/version, BLAKE3 identities, quota/backpressure policy, current-attempt binding, interruption point, resumed demand, transferred/reused byte accounting, output-admission result, and bounded non-claims.

#### Scenario: Current evidence supports a completion claim

- GIVEN the production remote path transfers an artifact larger than a control-frame payload through bounded chunks
- AND a current-attempt interruption/reconnect resumes by sending only verified missing content
- WHEN completion is reported
- THEN tracked evidence MUST cite the exact positive run plus negative stale-checkpoint, digest, quota, backpressure, and superseded-fence runs
- AND the claim MUST remain limited to the tested Mantle transfer modes and output-admission policy.

#### Scenario: Inline or contradictory evidence blocks completion

- GIVEN a task is checked complete but current evidence says resumable transfer was not implemented, or the production path still stores whole NAR/input payloads in frame vectors
- WHEN status, archive readiness, docs, or a final summary is produced
- THEN Mantle MUST state that production resumable transfer is incomplete or unproven and cite the contradiction
- AND a capability enum, unit fixture, broad accepted requirement, or stale checked task MUST NOT be promoted as implementation proof.
