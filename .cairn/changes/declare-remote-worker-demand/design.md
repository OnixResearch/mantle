# Design: Declare remote worker demand

## Goal and scope

Turn an existing, accurate placement blocker into a bounded published fact that
a supervisor may act on. Change nothing about how Mantle places or admits work.

## Current behavior

`crunch-build::distributed` plans placement against admitted worker
inventory. When no worker fits, the planner reports a quantified-resource
blocker. The blocker lives in the plan result and the build report. ADR 0025
prohibits replacing a missing capacity fact with an inferred one.

## Approach review

| Family | Mechanism | Disposition | Required check |
| --- | --- | --- | --- |
| Infer capacity from concurrency | Derive a worker count from job demand | Rejected by ADR 0025: fabricates capacity | Not blocking |
| Publish the blocker as demand | Bounded fact with the admitted requirement | Selected direction | Demand and retraction fixtures |
| Queue the job until capacity appears | Extend the wait with a durable queue entry | Deferred: changes scheduling semantics beyond this change | Placement fixture |
| Start workers from Mantle | Provisioning inside the build tool | Rejected: out of scope for a build tool (ADR 0010) | Not blocking |

## Contract and component ownership

- Pure core: demand derivation from admitted requirement and placement facts,
  requirement identity, deduplication, retraction decision, and bounds.
- Shell: surface publication and retraction, and the operator-facing report
  rendering.
- Supervisor: outside Mantle. It may read demand facts and start a worker. Its
  decisions re-enter Mantle only through ordinary worker registration.

## Implemented source boundary (execution not yet accepted)

`crates/crunch-remote-core/src/demand.rs` owns bounded canonical resource
vectors, requirement hashing, one-current-fact-per-job deduplication, and
explicit retraction. `src/remote_build/demand.rs` derives only from validated,
quantified pending placement facts. Neither interface accepts worker presence,
concurrency, or historical execution as a resource vector. Requirement identity
binds the job, default scheduling policy, quantities, named resources,
semantic classes, and normalized placement key; the blocker is reported
verbatim but does not change that requirement identity.

The book and daemon each cap demand at 1,024 facts. The daemon's global demand
cap is independent of the existing 4,096 all-fact cap. An existing fact may
update its blocker without consuming another slot, but publishing a changed
requirement for the same owned job before retracting its old fact is rejected.
The building-plane diff emits retractions before replacement publications.

Production admission temporarily represents a no-fit result as internal
`Pending` so the building plane can project its unmet requirement. It still
returns the original error immediately: no wait loop, provisioning, or queue
was added. The shared dispatch-preparation shell emits the complete bounded
demand diagnostic even without a daemon, queues its scoped publication, then
retracts it with `OwnerStopped` and queues the terminal diff. This is the entire
lifetime of a fail-fast request. Existing direct coordinator callers continue
to own longer pending, placed, cancelled, timed-out, and changed-requirement
lifetimes explicitly. Durable state omits the demand book.

Publication uses the existing bounded, nonblocking best-effort publisher. It
does not wait for daemon acknowledgement or join a socket worker. In a
long-lived adapter, queued publication and retraction drain in order; an
immediately exiting CLI may terminate its worker before delivery. A demand
diagnostic, queued write, or connected socket is not captured subscriber
evidence. Positive delivery must be observed with a real long-lived adapter;
daemon absence must retain the original blocker. No runtime acceptance is
inferred from the presence of source fixtures. Proposed ADR 0127 records these
decisions without accepting their execution gates.


## Decisions

### Decision: Demand restates admitted facts only

**Choice:** A demand fact carries the requirement vector, job identity, policy
identity, and blocker reason from the existing plan result. It adds no estimate.

**Rationale:** The requirement already exists. Publishing it cannot fabricate
capacity, and it keeps ADR 0025's refusal intact.

### Decision: Demand retracts, it does not expire

**Choice:** Demand retracts on placement, cancellation, timeout, or requirement
change. No TTL.

**Rationale:** A clock-based demand model would report stale demand after the
job ended. Lifetime-bound facts match the reviewed model.

### Decision: Demand cannot grant anything

**Choice:** A demand fact is not a lease, reservation, or placement.

**Rationale:** The supervisor's response must pass ordinary registration and
admission. This keeps trust and capacity checks in one place.

## Risks / Trade-offs

- A supervisor could over-provision. That is a supervisor policy decision, and
  Mantle reports demand without endorsing a response.
- Demand facts could churn under frequent requirement changes. Deduplication by
  requirement identity and count bounds contain it.
- If the coordination surface is absent, demand falls back to the plan result
  and build report.

## Non-Claims

- A demand fact proves a requirement, not the existence of capacity.
- A satisfied demand is not placement evidence or execution evidence.
