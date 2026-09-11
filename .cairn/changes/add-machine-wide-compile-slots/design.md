# Design: Add machine-wide compile slots

## Goal and scope

One per-machine authority bounds total compile concurrency across concurrent
Mantle builds. It composes with the compile-cache driver seam and shares its
trust framing: scheduling device, never evidence, outputs unchanged.

## Current behavior

The Worker bounds dispatch per build; nothing bounds machine-wide compile
concurrency across builds. Long proof sessions run several derivations in
parallel, each with internal parallelism, oversubscribing the host. pueue
bounds shell-level task parallelism but not compiles inside sandboxes.

The external reference implements slot grants in the cache daemon: cc and
rustc take one slot per run, Go through `-toolexec`, GHC through a semaphore
protocol served from slots (`evidence/repkgs-review.md`). Its design note
names the motivation: 384 sandboxes each running `make -j384` would
oversubscribe the machine.

## Approach review

| Family | Mechanism | Disposition | Required check |
| --- | --- | --- | --- |
| Per-build cores only | Current scheduling | Rejected as sufficient: cross-build oversubscription remains | Contention fixture |
| Global lock per compile | Serialize all compiles machine-wide | Rejected: destroys throughput | Grant-wait bound |
| Slot authority with policy | Bounded grants, per-build fairness, degradation | Selected direction | All four requirements' fixtures |
| cgroup/CPU limits at sandbox level | Kernel-level caps | Deferred: host-specific, opaque to receipts; may compose later | Not blocking |

## Contract and component ownership

- Pure core: grant policy evaluation (admission, fairness ordering, queue
  bounds) over in-memory request state; no clock, no I/O.
- Shell: the authority service in the existing daemon, the driver slot
  request, endpoint mapping, and scheduling receipts.
- Policy: typed Nickel export for slot policy, versioned.

## Decisions

### Decision: Same daemon as the compile cache

**Choice:** The slot authority is served by the cache daemon.

**Rationale:** The driver already talks to it; a second service would
duplicate endpoint mapping and lifecycle. If the cache is absent, the
authority can still run alone.

### Decision: Slots are advisory-capable

**Choice:** Builds always work without the authority.

**Rationale:** Scheduling must never become a build precondition; a dead
daemon must not kill proofs, matching the cache degradation contract.

## Risks / Trade-offs

- Slot waits add latency under contention; the fairness bound keeps waits
  observable and typed rejections prevent unbounded queues.
- Fairness policy needs real contention fixtures; synthetic single-build
  tests cannot prove starvation control.
