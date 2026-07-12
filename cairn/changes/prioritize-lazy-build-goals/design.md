## Context

`Worker` owns the imperative event loop and currently holds `ready_queue: VecDeque<String>`. Goals become ready when dependencies complete or when new roots/dynamic units are registered. The scheduler intentionally does not wait for evaluation to finish and cannot assume a complete graph. Route planning and remote capability matching already expose provider-neutral facts that can be normalized before scheduling.

## Decisions

### 1. Preserve lazy goals and replace only ready ordering

**Choice:** Keep the existing goal registry, waiter graph, streaming `want()` calls, `JoinSet`, semaphore, and dynamic-plan activation. Replace the FIFO ready queue with a deterministic priority-ready set keyed by stable goal identity.

**Rationale:** The missing feature is ordering, not graph ownership. An eager DAG would regress overlap and dynamic dependency support.

### 2. Use a named lexicographic priority tuple

**Choice:** The pure kernel returns an ordered tuple with named fields: operator policy class, starvation class, known critical-path pressure, resource-fit class, locality/transfer class, and stable realization-key tie-breaker. Policy may change field precedence explicitly, but Mantle does not use opaque weighted sums or unexplained numeric literals.

**Rationale:** Lexicographic fields are reviewable, deterministic, and easier to explain than a magic score.

### 3. Define critical path over the known graph

**Choice:** Compute a bounded lower-bound estimate from currently known dependency and waiter edges, optional policy-admitted duration classes, and blocked requested-root pressure. Recompute affected facts when goals, waiters, dynamic plans, or admitted history snapshots change. Reports label this `known-graph`, not global critical path.

**Rationale:** Lazy scheduling cannot honestly claim knowledge of future dynamic goals. The estimate can still prioritize work that currently unlocks long or highly shared chains.

### 4. Admit historical estimates only as explicit snapshots

**Choice:** Optional duration/resource history is canonicalized, bounded, BLAKE3-identified, and supplied as an input snapshot. Missing, stale, incompatible, or oversized history falls back to structural classes with a diagnostic; arrival timing never mutates priority implicitly.

**Rationale:** Replayability requires the scheduler's data dependencies to be visible.

### 5. Normalize resource and locality facts before ranking

**Choice:** Route/provider shells translate worker capacity, requested resources, input presence, and transfer estimates into bounded provider-neutral classes. The priority core sees no Kubernetes, S3, PostgreSQL, hostname, or live network API.

**Rationale:** Mantle's scheduler must stay provider-neutral and independently testable.

### 6. Prevent starvation with deterministic scheduling epochs

**Choice:** Every policy-defined ready-set event advances an explicit scheduling epoch. Waiting goals progress through bounded age classes, eventually outranking ordinary critical-path/locality preferences while still respecting hard eligibility and resource constraints.

**Rationale:** Wall-clock aging is difficult to replay and test; event epochs give a deterministic liveness policy.

### 7. Separate eligibility from preference

**Choice:** Route, capability, trust, upload privacy, and resource hard constraints are evaluated before priority. The comparator ranks only eligible dispatch candidates and cannot turn an ineligible worker/route into an eligible one.

**Rationale:** Scheduling preference must not bypass safety or output-admission policy.

### 8. Explain priority without leaking sensitive data

**Choice:** Build/status reports may include stable priority classes, known-path basis, history snapshot digest, age class, and tie-break class. They omit raw environment, bearer material, private paths, unbounded input lists, and provider credentials.

**Rationale:** Operators need replayable decisions, not sensitive scheduler internals.

## Functional Core / Imperative Shell

- **Core**: scheduling-fact validation, known-graph pressure propagation, age-class transition, priority tuple construction, total comparator, tie-break, and bounded diagnostics.
- **Shell**: graph mutation, queue updates, route/worker discovery, history loading, clocks used only for producing explicit snapshots, dispatch, cancellation, metrics, and rendering.

## Risks / Trade-offs

- Reprioritization has runtime cost; updates must be limited to affected known goals and bounded by named scheduler limits.
- Historical duration can reinforce stale behavior, so structural fallback and snapshot visibility are mandatory.
- Starvation promotion may modestly increase makespan but gives an explicit fairness bound.
- Priority evidence explains the selected order; it does not prove globally optimal scheduling.
