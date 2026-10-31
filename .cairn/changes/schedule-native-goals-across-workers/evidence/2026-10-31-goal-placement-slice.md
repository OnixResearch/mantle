# Goal placement slice (2026-10-31)

Scope: the pure placement decision S5 will call for each ready goal. It selects
among exactly the admitted pool endpoints, applies hard capability gates and
capacity, then a deterministic preference and stable identity. No adapter calls
it yet, so nothing here is dispatch, overlap, capacity, or execution evidence.

## Decision (`crunch_remote_core::pool::plan_goal_placement`)

Inputs: the `AdmittedPool` from `admits_pool`, one `GoalRequirement`, and the
adapter's current `WorkerFacts` per endpoint.

Denials, in order:

1. `NoAdmittedWorker` — no supplied facts name an endpoint of the admitted pool.
   Facts for endpoints outside the pool are ignored, so a pool label cannot
   select a worker.
2. `CapabilityMismatch` — system, sandbox mode, network mode, declared features,
   or declared accelerator classes fail a hard gate.
3. `InsufficientCapacity` — no compatible worker survives
   `resource::reserve_capacity`, or the goal needs named resources while the
   worker's named availability is constrained.

Selection: the caller supplies its scheduling preference order (the shell maps
`SchedulingPolicy.preference_order` onto `resource::PlacementPreference`), and
`resource::compare_placement` ranks the feasible candidates with the existing
stable `worker_endpoint_id` tie-break. The core does not choose a policy. The planner supplies no capacity and infers none
from concurrency; it returns the selected endpoint name and its worker
generation.

## Verification

| Gate | Result |
| --- | --- |
| `cargo test -p crunch-remote-core --lib pool` | 11 passed (7 admission + 4 placement) |
| `cargo clippy -p crunch-remote-core --all-targets --no-deps -- -D warnings` | exit 0 |
| `cargo fmt --check -p crunch-remote-core` | exit 0 |

Placement fixtures cover: two ready goals taking distinct workers once the first
reservation is applied, exhausted capacity denying rather than falling back, an
unadmitted endpoint being ignored, a hard capability gate denying, equal fit
breaking ties by stable endpoint identity, and a caller preference order changing
an equal-fit choice.

## Not claimed

- No live executor, reservation, transfer, or worker observation is involved.
- Placement facts remain outside semantic action identity; this module does not
  derive or alter that identity.
- Wiring into the lazy `Worker` and the remote application ports is still S5.
