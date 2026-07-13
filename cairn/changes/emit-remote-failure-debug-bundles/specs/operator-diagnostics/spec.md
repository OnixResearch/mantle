# Operator Diagnostics Specification

## Purpose

Define portable bounded remote-failure debug bundles and replay diagnostics.

## Requirements

### Requirement: Remote failures emit bounded content-bound debug bundles [r[operator_diagnostics.remote_failure_debug_bundle]]

Mantle MUST represent an operator-exported remote failure as a versioned bounded manifest over immutable refs for the action, route/assignment, worker capability summary, job/attempt/fence, declared inputs, sandbox/network policy, immutable log range when available, transfer/admission state, workspace mode, failure phase/reason, optional captured-artifact manifest, debug policy, and explicit non-claims. Mantle-owned bundle identity MUST use domain-separated BLAKE3. Bundle creation MUST NOT change execution, transfer, output-admission, or cleanup truth.

#### Scenario: Metadata-only bundle survives worker cleanup

- GIVEN a remote attempt fails and its declared action, route, policy, report, and available immutable log facts are durable
- WHEN Mantle exports a metadata-only debug bundle
- THEN the bundle MUST remain inspectable after the worker sandbox is removed
- AND it MUST identify missing optional evidence honestly rather than embedding mutable logs or fabricated refs.

#### Scenario: Secret-bearing facts are excluded

- GIVEN failure context includes bearer tickets, private key paths, raw secret or environment values, host paths, unbounded argv/log text, or captured bytes outside policy
- WHEN Mantle constructs or renders a bundle
- THEN it MUST omit, redact, or reject those facts according to typed policy
- AND the bundle MUST remain valid bounded machine data without exposing the protected content.

#### Scenario: Bundle validation fails closed

- GIVEN a bundle has a mismatched action/attempt/fence, malformed or stale ref, altered manifest digest, oversized list, unsupported schema, escaping path, or missing required immutable artifact
- WHEN Mantle validates it
- THEN validation MUST fail with stable diagnostics
- AND inspect or replay MUST NOT read unvalidated referenced content or mutate runtime state.

### Requirement: Failure replay is a new declared diagnostic execution [r[operator_diagnostics.remote_failure_replay]]

Mantle MUST inspect a failure bundle without execution and MUST construct replay as a new declared execution plan over validated immutable action/input/policy refs. Replay MUST receive a new route decision, job, attempt, and fence and MUST pass ordinary sandbox, network, transfer, output-admission, and action-result policy. Prior bundle or attempt authority MUST NOT authorize replay or output import.

#### Scenario: Inspect and replay-plan are side-effect free

- GIVEN a valid failure bundle is available on a clean host
- WHEN an operator requests inspect or replay-plan without execute
- THEN Mantle MUST return a bounded redacted summary or plan without starting a builder, opening a remote session, or mutating store/admission state
- AND the plan MUST identify unavailable inputs or policies as blockers.

#### Scenario: Replay uses new authority

- GIVEN an operator explicitly executes a valid replay plan
- WHEN Mantle schedules the replay locally or remotely
- THEN it MUST create new attempt/fence and route facts and enforce current policy
- AND stale worker credentials, old leases, old transfer checkpoints, or prior failure evidence MUST NOT authorize the new execution.

#### Scenario: Replay divergence remains diagnostic

- GIVEN replay exits differently, reaches another phase, emits different admitted outputs, or does not reproduce the original failure
- WHEN Mantle compares the executions
- THEN it MUST report bounded matching and divergent fact classes
- AND it MUST NOT rewrite the original result, claim replay-system failure solely from divergence, or claim the original failure was deterministic.
