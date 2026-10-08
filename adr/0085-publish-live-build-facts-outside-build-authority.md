# ADR 0085: Publish live build facts outside build authority

## Status

Proposed

## Context

ADR 0080 keeps coordination in a separate resident daemon. The existing
`--json build` aggregate and `--evaluation-stream` NDJSON report observations
but do not serve a current state with retractions. A worker can disappear
while a build still has remote coordinator jobs and resource leases. Consumers
need a bounded snapshot plus later changes, without making build success,
output admission, or evidence depend on daemon availability.

Molten's local dataspace primitives deliver an initial matching set and later
assert/retract events and clean up actor-owned assertions. The reviewed
primitive does not, by itself, establish Mantle's bounded multi-root filters,
subscriber quota, slow-consumer drop, or restart semantics.

## Decision Drivers

- Preserve existing aggregate and NDJSON schemas and build semantics.
- Retract facts when their owner or worker session stops being present.
- Distinguish a live coordinator belief from durable evidence and output trust.
- Bound every fact, observation set, filter, and subscriber queue.

## Decision

The building plane owns fact identity, bounded normalization of current
scheduler state, and change-driven publish/retract planning. A build emits
best-effort through a bounded non-blocking shell channel: a missing or failed
daemon MUST NOT reject a build or alter its outputs, receipts, or admission.
No heartbeat establishes liveness. An explicit current worker-session set,
not durable worker registration, is required for worker-presence assertions.
Worker loss retracts that presence and dependent reservations and dispatched
goals, while terminal outcome is an observation, not a successful receipt.

Local build dispatch is sourced from the actual `crunch-build::Worker` goal
registry after scheduling, not from selected-root evaluation records. Its
in-process worker presence lasts only for the build owner; local dispatch
has no remote attempt id, fence generation, or resource lease to assert.
The observation callback and channel never wait for socket I/O or determine
whether a build succeeds.

The coordination daemon owns subscriber state, initial matching snapshots,
subsequent publish/retract events, bounded filter validation, quotas, and
slow-subscriber drops. It does not mutate the store, admit outputs, or author
evidence. Molten may supply local assertion routing behind a narrow
Mantle-owned port only after its host supplies and proves the missing bounds
and lifecycle rules. The local Unix-socket daemon uses bounded in-memory
state for that port; choosing a provider never gives it evidence authority.

The evaluation stream's content-addressed run id stays unchanged. An opted-in
build mints a separate per-invocation live owner id in the imperative shell,
so concurrent runs of identical input do not share daemon ownership or
retract one another's assertions.

Facts belong to the emitting owner run and its current daemon incarnation.
An owner stop retracts all of its facts. Graceful daemon shutdown retracts
facts before disconnecting observers; a crash cannot deliver retractions.
Therefore a disconnect invalidates all facts previously observed on that
connection, and a restarted daemon begins with an empty current set until
live owners emit again. Old daemon facts MUST NOT reappear by persistence.

## Consequences

Remote coordinator state and actual in-process Worker transitions can be
projected into bounded, versioned pure facts independently of the daemon.
Output/receipt identity, daemon subscription, and subscriber/drop/restart
behavior remain distinct acceptance gates. Live facts prove current
coordination belief only, never output trust, build success, or release
eligibility.
