# Design: Add machine-wide compile slots

## Goal and scope

One per-machine authority bounds total compile concurrency across concurrent
Mantle builds. It composes with the compile-cache driver seam and shares its
trust framing: scheduling device, never evidence, outputs unchanged.

## Current behavior

The Worker still bounds dispatch per build. An explicitly enabled C/C++ driver
can now request machine-local grants from the existing cache daemon;
`crunch-rust-cache-core::slots` supplies pure bounded round-robin/FIFO policy
and `crunch-rustc-wrapper::slots` owns connection/child lifecycle observations.
No authority remains a supported advisory mode, not a global-bound claim.

Pure and process fixtures plus `scripts/prove-cc-cache-slots.py` cover queue
bounds, real compiler contention, client/authority loss and output controls.
Representative concurrent bootstrap builds and slot-on/off identity parity
remain unexecuted acceptance obligations; fixture source is not evidence.

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

## Grant admission and real contention controls

The client validates build/request identity, schema, reason, policy/resource
bounds, occupancy and grant wait before attaching compiler ownership to a
receipt. Invalid receipts degrade to ordinary compilation; they cannot expand
the supported scheduling limits. The daemon continues to independently own
kernel-translated child credentials, pidfd and cooperative process group.

The shared real-compiler harness holds compilers on source FIFOs rather than
using sleeps as evidence of concurrency, tests an already-saturated Unix
accept backlog, and kills the authority while a granted compiler is live.
It requires that live compiler and subsequent degraded compiles preserve
their real outcomes. Those checks are not yet captured execution and do not
replace the representative bootstrap-build contention/output gates.

