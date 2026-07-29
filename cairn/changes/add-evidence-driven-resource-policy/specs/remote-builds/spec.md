# Remote Builds Resource Policy Delta

## ADDED Requirements

### Requirement: remote_builds.resource_observations

r[remote_builds.resource_observations]

Mantle SHALL record bounded, versioned resource observations for terminal remote attempts and SHALL distinguish observations from declared requirements and future guarantees.

#### Scenario: Completed attempt has valid measurements

- **GIVEN** a terminal attempt and trusted bounded worker measurements
- **WHEN** Mantle commits the observation
- **THEN** it SHALL link public attempt, action-family, platform, machine-class, measurement, outcome, collector, and schema identities
- **AND** it SHALL omit secrets, token claims, environment values, and log content

#### Scenario: Observation is unusable

- **GIVEN** a missing, stale, incompatible, oversized, or untrusted observation
- **WHEN** selection policy filters history
- **THEN** it SHALL exclude that observation with a stable reason code
- **AND** it SHALL retain the declared resource minima

### Requirement: remote_builds.replayable_resource_selection

r[remote_builds.replayable_resource_selection]

Mantle SHALL select a machine class through a pure, deterministic, versioned policy over declared requirements, eligible classes, quota facts, and compatible observations.

#### Scenario: Compatible history supports selection

- **GIVEN** declared minima, eligible classes, available quota, and sufficient compatible observations
- **WHEN** the selector runs
- **THEN** it SHALL return one eligible class and ordered reason codes
- **AND** replay with the same canonical inputs and policy version SHALL return the same result

#### Scenario: History conflicts with hard requirements

- **GIVEN** observations that suggest a class without a required architecture, platform, KVM, trust, isolation, or minimum resource feature
- **WHEN** the selector runs
- **THEN** it SHALL reject that class
- **AND** history SHALL NOT weaken the hard requirement

### Requirement: remote_builds.positive_oom_retry

r[remote_builds.positive_oom_retry]

Mantle SHALL create a resource-escalation retry only after trusted positive OOM evidence and within named retry, machine-class, quota, cumulative-charge, and wall-time limits.

#### Scenario: OOM evidence allows escalation

- **GIVEN** a failed attempt with trusted positive OOM evidence and an eligible larger class within all limits
- **WHEN** the retry planner runs
- **THEN** it SHALL create a new fenced attempt linked to its predecessor
- **AND** it SHALL record the policy version and escalation reason

#### Scenario: Failure is not proven OOM

- **GIVEN** a non-OOM failure, ambiguous termination, or exit status without required platform evidence
- **WHEN** the retry planner runs
- **THEN** it SHALL NOT schedule an OOM escalation
- **AND** it SHALL return a terminal reason code

### Requirement: remote_builds.usage_reservation_and_reconciliation

r[remote_builds.usage_reservation_and_reconciliation]

Mantle SHALL reserve and reconcile project and account usage through idempotent ledger operations with explicit units, windows, policy versions, and attempt identities.

#### Scenario: Attempt completes once

- **GIVEN** an admitted reservation and bounded completion observation
- **WHEN** Mantle reconciles usage
- **THEN** it SHALL commit one charge under the active unit schedule
- **AND** it SHALL release or adjust the reservation as policy defines

#### Scenario: Completion is delivered twice

- **GIVEN** duplicate completion processing for the same attempt identity
- **WHEN** the ledger applies reconciliation again
- **THEN** it SHALL return the existing reconciliation
- **AND** it SHALL NOT double-charge usage

#### Scenario: Quota mutation fails

- **GIVEN** unavailable or inconsistent accounting state
- **WHEN** admission or reconciliation cannot commit safely
- **THEN** Mantle SHALL fail closed or enter an explicit recoverable state
- **AND** it SHALL NOT silently grant additional quota

### Requirement: remote_builds.authorized_result_sharing

r[remote_builds.authorized_result_sharing]

Mantle SHALL keep result discovery private by default and SHALL permit cross-project reuse only under an explicit sharing scope and all existing identity, signature, policy, platform, output, and CAS checks.

#### Scenario: Explicit consumer reuses a result

- **GIVEN** matching request identity, trusted producer signature, compatible policy and platform, authorized producer and consumer scopes, verified outputs, and current CAS availability
- **WHEN** result discovery evaluates reuse
- **THEN** it MAY return the signed result
- **AND** it SHALL record a producer-to-consumer evidence link

#### Scenario: Shared object lacks result authority

- **GIVEN** an output object present in shared CAS without one required signature, scope, policy, platform, identity, or availability fact
- **WHEN** a consumer requests reuse
- **THEN** Mantle SHALL reject result usability
- **AND** shared storage presence SHALL NOT imply shared authority

### Requirement: remote_builds.resource_benchmark_evidence

r[remote_builds.resource_benchmark_evidence]

Mantle SHALL maintain license-reviewed, fixed-input benchmark fixtures and SHALL report correctness, compatibility, performance, memory, transfer, and usage observations as separate bounded facts.

#### Scenario: Policy version is benchmarked

- **GIVEN** a declared static baseline and a versioned selection policy
- **WHEN** the benchmark corpus runs on a declared platform
- **THEN** the report SHALL identify sources, licenses, inputs, platform, policy versions, and measurement bounds
- **AND** Cairn SHALL be able to gate the recorded comparison

#### Scenario: Benchmark passes

- **GIVEN** passing benchmark observations
- **WHEN** evidence is reviewed
- **THEN** it SHALL NOT claim future resource sufficiency, production performance, host isolation, output correctness, or fair billing outside tested inputs

### Requirement: remote_builds.resource_policy_rollout

r[remote_builds.resource_policy_rollout]

Mantle SHALL support observe-only selection and independent feature controls for historical selection, OOM retry, quota enforcement, and cross-project sharing.

#### Scenario: Observe-only mode differs from static policy

- **GIVEN** a historical-policy decision that differs from the active static decision
- **WHEN** observe-only mode runs
- **THEN** Mantle SHALL record both decisions and reason codes
- **AND** it SHALL schedule with the static decision

#### Scenario: One feature is disabled

- **GIVEN** a rollback that disables OOM retry but leaves observation collection enabled
- **WHEN** an OOM attempt completes
- **THEN** Mantle MAY record the observation
- **AND** it SHALL NOT schedule an escalation retry
