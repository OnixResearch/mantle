# Remote Builds Specification

## Purpose

Define durable attempt identity, fencing, and pure decision boundaries for remote-build execution.

## Requirements

### Requirement: Remote assignments use durable attempt fencing [r[remote_builds.durable_attempt_fencing]]

Mantle MUST distinguish normalized realization keys, durable coordinator jobs, and per-assignment execution attempts. Every current assignment MUST carry a unique attempt id and a monotonically advancing fence generation, and every state-changing worker report MUST match the coordinator's current job, attempt, and fence before it can mutate durable state or reach output admission.

#### Scenario: Reassignment advances the fence

- GIVEN a live remote job is reassigned after timeout, worker loss, policy-directed retry, or operator cancellation
- WHEN the coordinator publishes the replacement assignment
- THEN it MUST durably record a new attempt id and a fence generation greater than the superseded assignment
- AND the superseded assignment MUST no longer be authorized to mutate current job state.

#### Scenario: Stale completion cannot win a race

- GIVEN an older worker reports completion after a replacement attempt became current
- WHEN the coordinator validates the older report's job, attempt, and fence
- THEN Mantle MUST reject the report as stale before transfer acceptance or output admission
- AND it MUST NOT replace, merge, or relabel the current attempt's state with the stale result.

#### Scenario: Restart preserves the current owner

- GIVEN a coordinator restarts with a queued, running, transferring, or finished-undelivered attempt
- WHEN durable state is reloaded
- THEN Mantle MUST recover the current attempt id, fence generation, phase, and retained-result disposition
- AND ambiguous or legacy live state without a safe current fence MUST fail closed instead of inventing successful ownership.

### Requirement: Attempt reports are idempotent and conflict detecting [r[remote_builds.idempotent_attempt_reporting]]

Mantle MUST apply attempt reports through stable event ids and canonical payload digests. Repeating the same event id with the same digest MUST be an idempotent no-op, while reusing an event id with different content or applying an event from a stale fence MUST fail before durable state changes.

#### Scenario: Duplicate delivery is harmless

- GIVEN a current attempt report was durably applied
- WHEN transport redelivers the same event id with the same canonical payload digest
- THEN Mantle MUST return an already-applied disposition
- AND job, log, transfer, lease, and output state MUST remain unchanged.

#### Scenario: Conflicting duplicate fails closed

- GIVEN a current or retained event id already binds one canonical payload digest
- WHEN a peer submits the same event id with a different payload digest
- THEN Mantle MUST reject the report with a stable conflict reason
- AND it MUST NOT apply either a merged event or the conflicting payload.

### Requirement: Remote attempt decisions have a pure deterministic core [r[remote_builds.pure_attempt_decisions]]

Mantle MUST implement remote authorization, retry, transition, idempotency, and fence decisions as pure deterministic functions over explicit bounded input facts. Clock reads, persistence, transport, cancellation, process control, cryptographic admission, and rendering MUST remain in imperative shell code.

#### Scenario: Equivalent facts produce equivalent decisions

- GIVEN two decision evaluations contain equivalent current attempt state, authorization facts, retry policy, failure class, budget, supplied time facts, and report identity
- WHEN the pure core evaluates them
- THEN both evaluations MUST return the same decision, next state, and stable reason code
- AND discovery order, wall-clock reads, environment state, filesystem state, and transport timing MUST NOT affect the result.

#### Scenario: Shell failure cannot bypass a core rejection

- GIVEN the pure core classifies a report as stale, unauthorized, conflicting, over budget, or terminal
- WHEN the coordinator shell handles that decision
- THEN the shell MUST NOT persist the rejected mutation or pass its output to admission
- AND a logging, cancellation, or exporter failure MUST NOT turn the rejection into success.
