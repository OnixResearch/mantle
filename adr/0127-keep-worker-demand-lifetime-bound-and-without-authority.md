# ADR 0127: Keep declared worker demand lifetime-bound and without authority

## Status

Proposed (2026-10-23). The pure demand model, admission projection, transient
socket relay, and fail-fast publication adapter exist in source. Runtime
publication/retraction, no-inference, and integration gates remain unaccepted
until their commands and results are captured. This does not accept or archive
the `declare-remote-worker-demand` change.

## Context

ADR 0025 forbids deriving quantified capacity from concurrency, presence, or
history. ADR 0080 separates the building plane's decisions from a daemon that
serves revocable coordination facts. A quantified placement blocker should be
visible to an external supervisor without changing that admission boundary.

## Decision Drivers

- Publish a concrete quantified blocker without inferring worker capacity.
- Bind transient demand to its live owner and retract it on owner loss.
- Preserve the build plane's placement, authority, and release decisions.

## Decision

Demand restates an admitted requirement and the actual placement blocker. It
contains the request/job identity, scheduling policy identity, exact resource
vector, semantic placement identity, requirement digest, and unchanged blocker
reason. The requirement digest binds job, policy, quantities, named resources,
semantic accelerator classes, and placement identity; it is not a worker fact.
Canonical vectors and identities are bounded before hashing or publication.

The building plane derives demand. The daemon validates its bounded shape and
identity and relays it; it never derives demand or capacity. The pure book
retains at most 1,024 current demands, one per job. The daemon also caps demand
facts at 1,024 globally, independently of its existing all-fact cap. Repeated
placement rounds replace or reuse a requirement rather than accumulating facts.
A publisher must retract its previous requirement before publishing a different
requirement for the same job. Count overflow refuses the new fact; it cannot
create a worker, reservation, lease, or placement.

Demand has the owning request's lifetime, not a TTL. Placement, cancellation,
timeout, requirement change, or owner termination retracts it. Lost publication
connections retract their facts; daemon restart starts an empty epoch. Durable
coordinator serialization omits the diagnostic book.

A fail-fast production admission may internally retain an unmet requirement
through its `Pending` decision only until the shell reports that failure. This
is not a queue: production returns the same blocker immediately, with no new
worker wait or retry loop. The shell renders the bounded demand even without a
daemon, queues its publication, retracts it with `OwnerStopped`, queues that
retraction, and returns the original error. Delivery uses the existing bounded
best-effort observation sink and is not a daemon acknowledgement. Daemon absence
or observation failure must not replace the placement blocker.

Demand cannot grant anything. Supervisors remain outside Mantle; workers they
start must register through ordinary inventory and admission. No demand consumer
may infer resources from concurrency, live presence, or historical execution,
change scheduling preference, or treat demand as execution or release evidence.

## Alternatives rejected

- Inferring capacity from concurrency or observed workers: violates ADR 0025.
- Provisioning workers inside Mantle: exceeds the build-tool boundary.
- Requiring the daemon or a durable pending queue: changes fail-fast admission.
- Expiring demand by a guessed TTL: outlives cancellation or requirement changes.
- Promoting live demand to receipts: confuses coordination with evidence.

## Verification boundary

Core demand fixtures cover identity, deduplication, replacement, explicit
lifetime ends, invalid vectors, and count refusal. Admission fixtures cover
registration without quantified inventory, no leases while blocked, and
ordinary inventory registration resolving placement. Socket fixtures cover
owner-loss retraction, forged identity, per-job replacement ordering, and the
global count bound. These are source fixtures, not captured passing results.
The actual fail-fast CLI/gateway publication and retraction must be exercised
with a real subscriber, with daemon absence as a control, before acceptance.
