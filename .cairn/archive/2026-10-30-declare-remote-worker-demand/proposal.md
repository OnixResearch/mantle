# Proposal: Declare remote worker demand

## Why

When a quantified job has no admitted worker that fits, Mantle reports a
resource blocker and waits. The blocker is accurate, but it is invisible to
anything that could add capacity. ADR 0025 correctly refuses to infer capacity
from concurrency, so Mantle must not guess. It can, however, publish the
requirement it already has.

The Synit manual starts services from declared need: a `require-service`
assertion exists while demand exists, and the service stops when the last
interest is withdrawn
(`~/.local/share/mantle-references/synit-book/pages/12-operation__service.md`,
reviewed in `docs/synit-application-notes.md`). Demand is an assertion, not a
capacity estimate.

Mantle already records the exact requirement: the admitted quantified resource
vector, the job, the attempt, and the placement blocker. Publishing that fact
lets an operator-side supervisor respond without Mantle inventing capacity.

## What Changes

- Publish a bounded demand fact when a quantified job cannot be placed under
  current admitted worker facts. The fact MUST name the required resource
  vector, the job identity, the policy identity, and the blocker reason.
  r[build_scheduling.declared_worker_demand]
- Retract the demand fact when the job is placed, cancelled, times out, or its
  requirement changes.
  r[build_scheduling.declared_worker_demand]
- Separate demand from capacity. A demand fact MUST NOT create a worker, a
  lease, a reservation, or a placement, and MUST NOT change admission or
  scheduling preference.
  r[build_scheduling.demand_without_capacity_inference]
- Deduplicate demand by requirement identity so repeated placement rounds do
  not multiply facts, and bound the number of live demand facts.
  r[build_scheduling.declared_worker_demand]
- Publish and retract demand through the coordination daemon from
  `publish-live-build-state-subscriptions`. The building plane derives the
  fact. The daemon serves it. A plan result and build report MUST keep the
  same blocker when the daemon is absent.

## Impact

- **Immediate consumer**: remote farm operators and any supervisor that starts
  workers in response to declared need. The supervisor reads demand from the
  coordination daemon (ADR 0080).
- **Immediate outcome**: capacity starvation becomes a visible, addressable
  fact instead of a silent wait.
- **Durable capability**: a demand model that later covers cache, sandbox, and
  compile-slot resources.
- **Maintenance owner**: Mantle scheduling owner.
- **Repeatability evidence**: unmet-demand fixtures, retraction fixtures for
  placement and cancellation, deduplication fixtures, and a proof that demand
  never fabricates capacity.
- **Compatibility**: placement, admission, and reports keep their current
  behavior. Demand is additive.

## Scope

The change covers demand computation from realized placement facts, demand
publication and retraction, deduplication, bounds, and the supervisor boundary.

## Out of Scope

- Starting, stopping, or provisioning workers inside Mantle.
- Inferring capacity from concurrency, worker presence, or history.
- Autoscaling policy, provider selection, or cost decisions.
- Treating a satisfied demand fact as placement evidence.

## Success Criteria

- A quantified job with no fitting admitted worker publishes exactly one
  demand fact.
- The fact retracts when the job is placed or cancelled.
- A registered worker that satisfies the vector resolves the demand without
  Mantle creating a lease.
- Demand never changes placement order or admission outcomes.
