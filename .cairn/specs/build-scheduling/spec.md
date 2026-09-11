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

### Requirement: Quantified resources are admitted before scheduling preference
r[build_scheduling.quantified_resource_admission]

Mantle MUST validate provider-neutral quantified action requirements and worker capacities for CPU, memory bytes, scratch bytes, accelerator classes/counts, and bounded named token pools before route preference. It MUST use checked arithmetic and current fenced reservations to determine hard resource eligibility. Dynamic capacity and scheduling-only quantities MUST NOT change action identity unless an explicit semantic action field declares that they affect execution meaning.

#### Scenario: Compatible capacity is reservable

- GIVEN an eligible action declares bounded resource quantities and a worker has matching unreserved capacity and semantic capabilities
- WHEN Mantle plans resource admission
- THEN it MUST return a deterministic reservation plan and normalized resource-fit class
- AND applying that plan MUST leave every remaining capacity nonnegative and within declared totals.

#### Scenario: Aggregate overcommit is rejected

- GIVEN individually valid jobs collectively request more CPU, memory, scratch, accelerator, or named-token capacity than remains
- WHEN Mantle plans another assignment
- THEN it MUST reject that worker before dispatch with a stable resource reason
- AND scheduler priority, locality, age, or provider response order MUST NOT authorize overcommit.

#### Scenario: Scheduling metadata does not silently alter action identity

- GIVEN two executions differ only in scheduling-only reservation quantities or dynamic worker availability
- WHEN Mantle computes their action refs
- THEN those facts MUST NOT change action identity
- AND any platform, toolchain, accelerator, sandbox, or other semantic difference MUST remain an explicit action-identity input.

### Requirement: Locality placement uses receiver-verified content facts
r[build_scheduling.verified_locality_placement]

Mantle MUST derive content-locality and transfer-cost preference from bounded receiver-verified object-presence and missing-set facts bound to the requested input manifest and policy. Unverified worker advertisements, stale probes, expected CA paths, or mutable cache claims MUST NOT establish full locality or zero transfer. Locality MAY rank eligible workers but MUST NOT bypass capability, trust, upload privacy, network, store-prefix, or hard resource policy.

#### Scenario: Verified local content improves placement

- GIVEN two workers satisfy every hard eligibility check
- AND one receiver verifies a more complete demanded input set with a lower bounded transfer class for the same manifest and policy
- WHEN the existing scheduler ranks the eligible candidates
- THEN it SHOULD prefer that worker according to configured field precedence
- AND evidence MUST identify normalized verified counts/bytes and freshness basis without unbounded object lists.

#### Scenario: Stale locality is rejected

- GIVEN a locality observation names the wrong manifest, policy, worker generation, or content that receiver probes no longer verify
- WHEN Mantle derives scheduling facts
- THEN it MUST reject or downgrade that observation with a stable reason
- AND it MUST NOT report `FullyPresent` or zero transfer from the stale claim.

#### Scenario: Locality cannot create eligibility

- GIVEN a worker has all input content but lacks output trust, upload permission, required semantic capability, store-prefix match, or reservable resources
- WHEN placement runs
- THEN the worker MUST remain ineligible
- AND no locality or transfer preference may cause dispatch.

### Requirement: Concurrency policy uses explicit parallelism facts [r[build_scheduling.explicit_parallelism_facts]]

Mantle MUST compute the build job limit from explicit requested jobs, observed available parallelism, configured policy cap, and applicable executor limit. The deterministic policy MUST NOT call host, environment, clock, provider, or async-runtime observation APIs.
#### Scenario: User supplies a valid job count [r[build_scheduling.explicit_parallelism_facts.scenario.user-supplies-a-valid]]

- GIVEN requested jobs, observed host parallelism, policy cap, and executor limit are valid bounded values
- WHEN concurrency policy selects the effective job count
- THEN it MUST return the same bounded count for equivalent facts
- AND insertion order, host timing, or ambient runtime state MUST NOT affect the result.

#### Scenario: Host observation is unavailable [r[build_scheduling.explicit_parallelism_facts.scenario.host-observation-is-unavailable]]

- GIVEN the user did not request a job count and the shell cannot obtain valid available-parallelism facts
- WHEN concurrency planning runs
- THEN it MUST return the declared fallback or a typed blocker according to policy
- AND the core MUST NOT query the host directly.

#### Scenario: Parallelism facts exceed bounds [r[build_scheduling.explicit_parallelism_facts.scenario.parallelism-facts-exceed-bounds]]

- GIVEN requested, observed, policy, or executor values are zero where forbidden, exceed named limits, or cannot convert safely
- WHEN concurrency policy validates them
- THEN it MUST fail or clamp only as the explicit policy declares
- AND it MUST use checked arithmetic and a stable reason code.
