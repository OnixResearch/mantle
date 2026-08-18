# Verification Evidence Specification

## Purpose

Adds Oxide-inspired release repository, ephemeral worker, and async cancellation-safety evidence profiles for Mantle.

## Requirements

### Requirement: Oxide release and worker reference inventory
r[mantle.verification_evidence.oxide_release_worker.reference_inventory] Mantle MUST record reference-only intake notes for each Oxide release, worker, or cancellation-safety repository used to shape Mantle evidence contracts.

#### Scenario: Reference intake records ownership boundary
r[mantle.verification_evidence.oxide_release_worker.reference_inventory.boundary]
- GIVEN Mantle evaluates an Oxide repository as prior art
- WHEN the intake record is written
- THEN it MUST name the source repository, intended adaptation, license posture, trust boundary, and non-claim boundary.

#### Scenario: Reference source is not proof authority
r[mantle.verification_evidence.oxide_release_worker.reference_inventory.non_authority]
- GIVEN an Oxide repository implements a similar release or worker pattern
- WHEN Mantle records it as a reference
- THEN Mantle MUST NOT treat that repository as proof of Mantle release correctness, reproducibility, or build soundness.

### Requirement: TUF-style release bundle profiles
r[mantle.verification_evidence.oxide_release_worker.tuf_bundles] Mantle SHOULD define TUF-style release repository profiles for signed metadata, target manifests, artifact tags, compatibility checks, and trust-root handling.

#### Scenario: Signed metadata binds targets
r[mantle.verification_evidence.oxide_release_worker.tuf_bundles.signed_targets]
- GIVEN a release bundle contains metadata and target artifacts
- WHEN Mantle verifies the bundle under a TUF-style profile
- THEN signed metadata SHOULD bind each target artifact identity, role, expiration policy, and compatibility profile.

#### Scenario: Artifact tag mismatch fails closed
r[mantle.verification_evidence.oxide_release_worker.tuf_bundles.tag_mismatch]
- GIVEN a target artifact's typed tags do not match its declared release role
- WHEN Mantle verifies release metadata
- THEN verification MUST fail closed with deterministic diagnostics.

### Requirement: Ephemeral worker evidence profiles
r[mantle.verification_evidence.oxide_release_worker.ephemeral_workers] Mantle MUST treat ephemeral build workers as bounded evidence producers with explicit job, artifact, log, and cleanup receipts.

#### Scenario: Worker success emits complete receipt
r[mantle.verification_evidence.oxide_release_worker.ephemeral_workers.complete_receipt]
- GIVEN an ephemeral worker completes a build job successfully
- WHEN Mantle records the worker result
- THEN the receipt MUST include input identity, target profile, worker identity, log identity, artifact identities, cleanup outcome, and replayable job event identity.

#### Scenario: Untrusted policy override is rejected
r[mantle.verification_evidence.oxide_release_worker.ephemeral_workers.policy_override]
- GIVEN a candidate input tries to override trusted worker policy
- WHEN Mantle plans the job
- THEN Mantle MUST reject the override or require explicit trusted-policy approval before execution.

### Requirement: Cancellation-safe worker orchestration
r[mantle.verification_evidence.oxide_release_worker.cancel_safety] Mantle MUST model async cancellation as an explicit worker outcome that preserves cleanup and evidence integrity or fails closed.

#### Scenario: Cooperative cancellation preserves cleanup
r[mantle.verification_evidence.oxide_release_worker.cancel_safety.cleanup]
- GIVEN a worker session receives a cancellation request
- WHEN the session reaches a cancellation point
- THEN Mantle MUST attempt configured cleanup and emit a final receipt that states cancellation, cleanup outcome, and artifact disposition.

#### Scenario: Mid-upload cancellation fails closed
r[mantle.verification_evidence.oxide_release_worker.cancel_safety.mid_upload]
- GIVEN cancellation interrupts artifact upload or receipt persistence
- WHEN Mantle classifies the outcome
- THEN Mantle MUST mark the artifact unaccepted unless integrity and final receipt persistence are both proven.

### Requirement: Fixture-backed validation
r[mantle.verification_evidence.oxide_release_worker.validation] Mantle MUST validate release repository, ephemeral worker, and cancellation-safety profiles with positive and negative fixtures before implementation adoption.

#### Scenario: Positive fixtures cover accepted paths
r[mantle.verification_evidence.oxide_release_worker.validation.positive]
- GIVEN valid signed metadata, artifact tags, worker receipts, and cooperative cancellation examples are present
- WHEN the fixture suite runs
- THEN accepted cases MUST produce stable Mantle evidence reports.

#### Scenario: Negative fixtures cover rejection paths
r[mantle.verification_evidence.oxide_release_worker.validation.negative]
- GIVEN expired metadata, tag mismatch, untrusted policy override, cleanup failure, mid-upload cancellation, or overclaim text are present
- WHEN the fixture suite runs
- THEN invalid cases MUST fail with deterministic diagnostics and no accepted release evidence.
