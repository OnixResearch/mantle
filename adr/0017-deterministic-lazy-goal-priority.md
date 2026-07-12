# ADR 0017: Prioritize lazy ready goals with deterministic known facts

## Status

Accepted

## Context

ADR 0001 selected lazy goals so Mantle can overlap evaluation and builds, deduplicate shared derivations, notify waiters, and admit dynamic goals without constructing an eager global DAG. The original `Worker` nevertheless dispatched eligible ready goals in FIFO insertion order. FIFO made discovery and response timing an accidental policy, ignored known path pressure and locality, and could indefinitely delay an ordinary goal under recurring preferred arrivals.

Mantle cannot honestly compute a global critical path: evaluation may still be streaming and native dynamic plans may add future goals. Remote/provider state also cannot become an implicit comparator input without breaking replayability.

## Decision Drivers

- Preserve the lazy `GoalRegistry`, waiter notification, dynamic insertion, streaming evaluation, and `max_jobs` dispatch limit.
- Make equivalent explicit facts produce one total order independent of insertion or map iteration.
- Keep hard capability, trust, upload, network, store-prefix, and resource constraints outside preference ranking.
- Bound graph traversal, history admission, arithmetic, evidence, and fairness state.
- Explain decisions without exposing environment values, credentials, bearer material, or private paths.
- Describe known facts without claiming global makespan optimality or future-graph knowledge.

## Decision

Mantle replaces only the FIFO ready queue with a deterministic ready set. `crunch-build::scheduling` is the functional core; `Worker` remains the imperative shell.

For each dispatch snapshot, the core validates a bounded immutable graph, optional duration-class history, policy, ready facts, and provider-neutral eligible preference classes. Invalid or oversized facts fail before the ready set is mutated. Known path pressure propagates from currently requested roots over currently known dependency/waiter edges. Graph changes invalidate only affected connected components. Missing, stale, incompatible, oversized, or unknown-goal history falls back to structural duration classes and records a stable fallback basis.

The default lexicographic order is:

1. operator policy class;
2. deterministic starvation class;
3. known-graph blocked-root count and critical-path work/node lower bounds;
4. provider-neutral resource-fit class;
5. verified content-locality and transfer-cost classes;
6. stable goal identity.

The three preference groups after starvation are explicitly ordered by `SchedulingPolicy.preference_order`; Mantle does not use weighted scores. Scheduling epochs advance on ready-goal selections, not wall-clock time. A continuously ready eligible goal progresses from `fresh` to `aged` to `protected`, so age eventually outranks ordinary graph/resource/locality preference within its operator policy class. Age never changes hard eligibility.

`BuildConfig` carries the validated runtime policy. `lib/scheduling.ncl` exposes the matching typed Nickel contract, defaults, graph bounds, and structural history fallback. The Rust and Nickel defaults are parity-tested.

Every selected goal produces one bounded `mantle-priority-decision-v1` row. Machine-readable build reports expose all rows under `scheduler_priority_decisions`; verbose human output renders a bounded prefix. Rows contain a BLAKE3 goal-identity digest, a BLAKE3 digest of the ordered redacted candidate snapshot, a redacted runner-up tuple, policy identity/digest, epoch and age class, known-graph/history basis, normalized preference classes, stable reason codes, claim scope, and explicit non-claims. They do not contain raw goal paths or provider secrets.

## Alternatives Considered

### Keep FIFO and document arrival order as policy

Rejected because asynchronous discovery timing is not a stable operator policy and provides no fairness or known-path preference.

### Construct an eager global DAG and schedule its critical path

Rejected because future streamed and dynamic goals are unknowable and ADR 0001 intentionally overlaps evaluation with execution.

### Use a weighted numeric priority score

Rejected because weights hide precedence, invite magic numbers, complicate overflow analysis, and make explanations less reviewable than named lexicographic fields.

### Age by wall-clock duration

Rejected because clock reads and scheduling jitter make replay and property testing nondeterministic.

### Let provider adapters rank candidates directly

Rejected because provider response order could bypass hard policy or become an ambient control dependency. Adapters may only emit bounded provider-neutral classes after all hard checks pass.

## Consequences

- Ready ranking has bounded overhead compared with FIFO. Pressure recomputation is component-scoped and ranking is snapshot-based; benchmark evidence is comparative and does not claim globally optimal scheduling.
- Newly admitted dynamic facts affect only later snapshots. Earlier evidence remains immutable and is never relabeled as though future graph facts were known.
- History can improve critical-path work estimates only when supplied as a compatible explicit snapshot; structural fallback remains deterministic.
- Operators can replay and explain configured ordering from JSON evidence, but the evidence does not prove execution success, output trust, release reproducibility, future graph completeness, or globally optimal makespan.
- Provider integrations must preserve the eligibility-before-preference boundary when adding new resource or locality facts.
