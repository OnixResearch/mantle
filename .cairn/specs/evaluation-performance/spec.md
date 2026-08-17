# Evaluation Performance Specification

## Purpose

Defines the `evaluation-performance` capability.

## Requirements

### Requirement: Typed evaluation budget policy

r[evaluation_performance.budget_policy] Mantle MUST admit evaluation through a typed, versioned policy with named bounds for source, imports, roots, diagnostics, protocol messages, wall time, CPU time, resident memory, workers, and teardown. Strict policy MUST fail before evaluation when the host cannot enforce a required mechanism.

#### Scenario: Host supports the strict policy

- **GIVEN** a valid request, supported worker isolation, and enforceable requested time and memory mechanisms
- **WHEN** Mantle admits strict evaluation
- **THEN** it MUST bind the request and policy identities before worker launch
- **AND** every configured limit MUST have a named unit and positive bounded value

#### Scenario: Strict memory enforcement is unavailable

- **GIVEN** policy requires an enforced resident-memory boundary but the host provides only observation or no supported mechanism
- **WHEN** evaluation admission checks platform support
- **THEN** Mantle MUST reject with `evaluation-budget-unsupported` before worker launch
- **AND** it MUST NOT present RSS sampling as enforcement

### Requirement: Bounded evaluator worker protocol

r[evaluation_performance.worker_protocol] Strict evaluation MUST use a versioned bounded worker protocol over explicit source, import, root, evaluator, and policy facts. Protocol decoding MUST reject malformed, oversized, duplicate, trailing, or incomplete data before disallowed allocation or evaluation.

#### Scenario: Valid request completes

- **GIVEN** one valid bounded request and a compatible evaluator worker
- **WHEN** the worker evaluates the selected roots
- **THEN** it MUST return one terminal response with result or error class, bounded diagnostics, observations, and worker identity
- **AND** the response MUST bind the request and policy identities

#### Scenario: Frame exceeds policy

- **GIVEN** a request or response declares a size above the named protocol bound
- **WHEN** the receiver reads the frame header
- **THEN** it MUST reject before allocating the declared payload
- **AND** it MUST not start evaluation or accept a partial response

### Requirement: Evaluation resource observations

r[evaluation_performance.resource_observations] Mantle MUST report bounded evaluation wall time, peak RSS when supported, CPU time when supported, source and import counts, discovered and selected roots, diagnostics, explicit force requests, terminal disposition, policy identity, and measurement support classes.

#### Scenario: Supported observations are recorded

- **GIVEN** an evaluator worker completes and the parent obtains supported process observations
- **WHEN** Mantle constructs the terminal report
- **THEN** it MUST record each value with an explicit unit and observation source
- **AND** the report MUST distinguish observed, enforced, unavailable, and truncated facts

#### Scenario: Peak RSS is unavailable

- **GIVEN** the host cannot provide an accepted peak-RSS observation
- **WHEN** observe-only evaluation completes
- **THEN** the report MUST mark peak RSS unavailable with a stable reason
- **AND** it MUST not emit zero or an estimated value as observed peak RSS

### Requirement: Metric roles remain separate

r[evaluation_performance.metric_role_separation] Mantle MUST distinguish public evaluation request counts, evaluator-provided internal observations, and parent process resource observations. Explicit root force requests MUST NOT be described as complete Nickel thunk or deep-export counts.

#### Scenario: One selected root is requested

- **GIVEN** Mantle explicitly requests one root and the evaluator can internally force or export additional values
- **WHEN** metrics are reported
- **THEN** `explicit_top_level_root_force_count` MAY report one request
- **AND** actual nonselected evaluation MUST remain absent unless an honest evaluator observation supplies it

#### Scenario: Metric role is overclaimed

- **GIVEN** docs, reports, or benchmark comparison label explicit API requests as complete internal thunk observations
- **WHEN** metric-contract validation runs
- **THEN** it MUST reject the overclaim
- **AND** it MUST preserve the narrower explicit-request meaning

### Requirement: Owned cancellation and teardown

r[evaluation_performance.enforced_teardown] Evaluation timeout or cancellation MUST end in one owned terminal outcome that records request, grace, termination, kill, reap, response, and cleanup facts. A late success response MUST NOT replace timeout or cancellation.

#### Scenario: Worker exceeds deadline

- **GIVEN** a worker remains active beyond the configured wall-time deadline
- **WHEN** the parent enforces timeout
- **THEN** it MUST terminate or kill and reap the owned worker within named teardown policy
- **AND** it MUST report timeout without accepting a late success response

#### Scenario: Worker cannot be reaped

- **GIVEN** termination begins but the parent cannot confirm worker reap within policy
- **WHEN** terminal classification runs
- **THEN** the outcome MUST report incomplete teardown and fail closed
- **AND** it MUST not report clean cancellation or successful evaluation

### Requirement: Cohort-bound benchmark gates

r[evaluation_performance.benchmark_gates] Evaluation benchmark comparison MUST bind host, target, evaluator, toolchain, policy, fixture, repeat, warm-state, and measurement-support facts. Threshold gates MUST apply only to compatible cohorts and MUST preserve missing metrics as missing.

#### Scenario: Compatible resource regression exceeds policy

- **GIVEN** baseline and candidate bundles have compatible cohort facts and one shared metric exceeds its named threshold
- **WHEN** benchmark comparison runs
- **THEN** it MUST report the regression with baseline, candidate, absolute, and percentage values
- **AND** policy MAY fail the benchmark gate for that metric

#### Scenario: Cohorts differ

- **GIVEN** baseline and candidate differ in a required cohort or peak-RSS observation mechanism
- **WHEN** benchmark comparison runs
- **THEN** it MUST report the incompatibility
- **AND** it MUST not apply a pass or fail threshold as if the values were directly comparable

### Requirement: Evaluation rollout preserves bounded parity

r[evaluation_performance.rollout] Mantle MUST run bounded positive and negative parity fixtures before the evaluator worker becomes authoritative. Unexplained output, error-class, diagnostic, root, or request-observation drift MUST block cutover while a bounded rollback remains available.

#### Scenario: Parity cycle agrees

- **GIVEN** current and worker paths consume equivalent bounded fixture facts
- **WHEN** one complete positive and negative parity cycle runs
- **THEN** accepted result and failure classes MUST agree under the declared comparison policy
- **AND** worker authority MAY advance only after recorded review

#### Scenario: Performance differs without semantic drift

- **GIVEN** paths agree on accepted result and failure facts but differ in time or memory observations
- **WHEN** parity is classified
- **THEN** Mantle MUST report the performance difference separately
- **AND** it MUST not classify performance difference alone as evaluator semantic equivalence or drift

### Requirement: Evaluator resource validation

r[evaluation_performance.validation] Evaluator resource work MUST include positive, negative, timeout, memory, crash, protocol, cancellation, teardown, metric-role, cohort, and missing-observation fixtures plus focused core, shell, benchmark, and lifecycle checks.

#### Scenario: Supported evaluation matrix passes

- **GIVEN** valid small, selected-root, import, evaluator-error, budget, cancellation, and benchmark fixtures
- **WHEN** focused validation runs
- **THEN** every fixture MUST produce its expected terminal class and bounded observations
- **AND** no worker process may remain after the test completes

#### Scenario: Negative fixture fails for its target boundary

- **GIVEN** a fixture has an oversized frame, timeout, memory exhaustion, crash, output flood, late response, failed reap, unsupported policy, metric overclaim, or cohort mismatch
- **WHEN** focused validation runs
- **THEN** it MUST produce the expected stable class
- **AND** an unrelated process or evaluator failure MUST NOT count as correct rejection evidence
