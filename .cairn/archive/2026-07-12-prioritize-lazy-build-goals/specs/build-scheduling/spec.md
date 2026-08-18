# Build Scheduling Specification

## Purpose

Define deterministic critical-path, resource, locality, and fairness ordering for Mantle's lazy ready goals.

## Requirements

### Requirement: Ready-goal priority is a pure deterministic total order [r[build_scheduling.deterministic_priority_kernel]]

Mantle MUST rank eligible ready goals through a pure deterministic comparator over explicit bounded scheduling facts and named priority fields. Equivalent facts MUST produce the same total order regardless of insertion order, map iteration, asynchronous response timing, wall-clock reads, random ids, or ambient provider state.

#### Scenario: Equivalent ready sets replay identically

- GIVEN two scheduler runs contain equivalent eligible goals, graph facts, policy, history snapshot, resource classes, locality classes, and age classes in different insertion orders
- WHEN the priority kernel ranks them
- THEN both runs MUST return the same dispatch order and stable tie-break reasons
- AND the final tie-break MUST use stable goal or realization identity rather than discovery timing.

#### Scenario: Invalid facts fail before queue mutation

- GIVEN scheduling facts are malformed, oversized, arithmetically invalid, or reference an unknown goal or policy class
- WHEN Mantle validates them
- THEN it MUST reject the facts with a deterministic diagnostic before changing the ready set
- AND it MUST NOT silently substitute wall-clock, random, or provider-response order.

### Requirement: Critical-path priority is bounded to the known lazy graph [r[build_scheduling.lazy_known_critical_path]]

Mantle SHOULD prioritize ready goals that unlock longer or more highly shared paths to currently requested roots by using bounded dependency/waiter facts from the graph known at that scheduling epoch. It MUST preserve streaming evaluation and dynamic goal insertion and MUST label the estimate as known-graph rather than global critical path.

#### Scenario: Known long path is preferred

- GIVEN two eligible ready goals have equal policy and starvation class
- AND one goal lies on a longer known path or blocks more currently requested roots
- WHEN Mantle constructs their priority tuples
- THEN it SHOULD rank that goal ahead according to the declared known-graph policy
- AND the priority evidence MUST identify the bounded path/blocked-root basis.

#### Scenario: Dynamic goal updates only known facts

- GIVEN a running build admits a valid dynamic plan that adds new goals and waiter edges
- WHEN the scheduler updates priority facts
- THEN Mantle MUST recompute affected known-graph pressure without rebuilding an eager global DAG
- AND prior reports MUST NOT be relabeled as if the future dynamic graph had been known earlier.

### Requirement: Resource and locality facts influence preference but not eligibility [r[build_scheduling.resource_locality_preference]]

Mantle SHOULD rank eligible goals using explicit provider-neutral resource-fit and content-locality/transfer classes. Priority MUST NOT make a route or worker eligible when capability, trust, upload privacy, network, store-prefix, or hard resource policy rejected it.

#### Scenario: Local compatible input improves preference

- GIVEN two otherwise equivalent eligible dispatch candidates satisfy all hard policy
- AND one candidate already verifies more demanded input content or has a better declared resource-fit class
- WHEN Mantle ranks them
- THEN it SHOULD prefer that candidate according to configured field precedence
- AND the report MUST identify the normalized class without exposing provider credentials or unbounded input lists.

#### Scenario: Priority cannot bypass a blocker

- GIVEN a candidate has high critical-path or locality preference but lacks required capability, output trust, upload permission, store-prefix match, or hard resource fit
- WHEN dispatch is planned
- THEN Mantle MUST keep that candidate ineligible
- AND no priority value or age promotion may convert the blocker into dispatch approval.

### Requirement: Continuously ready eligible goals have deterministic starvation protection [r[build_scheduling.deterministic_starvation_bound]]

Mantle MUST advance ready-goal age through explicit scheduling epochs and policy-bound age classes so a continuously ready eligible goal eventually outranks ordinary critical-path, resource, or locality preference. Age MUST NOT override hard eligibility or resource constraints.

#### Scenario: Waiting eligible goal is promoted

- GIVEN an eligible goal remains continuously ready while other ordinary-priority goals repeatedly enter the ready set
- WHEN policy-defined scheduling epochs advance
- THEN the waiting goal MUST progress through bounded age classes and eventually become dispatch-preferred
- AND the promotion point MUST replay identically from the same event sequence and policy.

#### Scenario: Ineligible goal does not age into dispatch

- GIVEN a goal is blocked on dependencies or lacks a hard capability, trust, upload, network, store-prefix, or resource requirement
- WHEN scheduling epochs advance
- THEN Mantle MUST NOT treat age as eligibility
- AND the blocker MUST remain visible until its explicit facts change.

### Requirement: Priority decisions emit bounded explanatory evidence [r[build_scheduling.priority_decision_evidence]]

Mantle MUST expose bounded human and machine-readable evidence for scheduler priority decisions when requested. Evidence MUST identify policy identity, known-graph/history basis, priority field classes, age class, selected goal, and stable tie-break class, while preserving scheduler and build non-claims.

#### Scenario: Operator can explain a dispatch

- GIVEN multiple eligible ready goals compete for an available slot
- WHEN Mantle records the selected dispatch
- THEN the report MUST include enough normalized priority facts to replay the ordering
- AND it MUST omit bearer material, private paths, raw environment values, provider credentials, and unbounded content lists.

#### Scenario: Priority evidence does not claim optimality

- GIVEN a deterministic priority report exists
- WHEN status or validation summarizes it
- THEN the claim MUST be limited to the configured known-fact ordering
- AND it MUST NOT claim globally optimal makespan, future-graph knowledge, successful execution, output trust, or release reproducibility.
