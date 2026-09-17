# Design: Live build state for coordination subscribers

## Goal and scope

One coordination daemon serves current build facts and retracts them when they
stop being true. The building plane emits facts and keeps its determinism and
its independence from the daemon.

## Current behavior

`--json build` emits one aggregate report at the end. `--evaluation-stream`
emits selected-root events as NDJSON during evaluation and build. Neither
supports subscription, and neither retracts a fact when a worker dies.

ADR 0080 assigns coordination to a separate resident daemon. Mantle already
runs daemon-shaped processes for the Rust cache and remote serve, so the
component boundary is familiar.

## Approach review

| Family | Mechanism | Disposition | Required check |
| --- | --- | --- | --- |
| Extend the NDJSON stream | Add subscribe/snapshot records | Rejected: a stream is not a state view; retraction stays implicit | Worker-loss fixture |
| Poll the aggregate report | Repeated `--json` runs | Rejected: expensive and stale between polls | Subscription-order fixture |
| Fold subscriptions into the build process | Long-lived build serving consumers | Rejected by ADR 0080: consumers would depend on a resident build | Daemon-absence fixture |
| Coordination daemon over Molten dataspaces | Build emits facts; daemon serves and retracts | Selected direction | Molten contract review plus all fixtures |

## Contract and component ownership

- Building plane (pure core): fact normalization, fact identity, bounded fact
  and event admission, and the mapping from scheduler state to emit actions.
- Building plane (shell): emission to the daemon or to a bounded local
  channel. Emission failure is non-fatal.
- Coordination daemon (pure core): subscription filter validation, current
  matching set, and retraction decisions over in-memory facts.
- Coordination daemon (shell): surface lifecycle, subscriber handling, and
  rendering. The daemon never blocks a build on a subscriber.
- Provider: the Molten dataspace component when its contract fits; otherwise a
  narrow port with the same semantics. The evaluation is task T1.3.

## Decisions

### Decision: Emission is best-effort and never corrective

**Choice:** A build emits facts when it can. A missing or failed daemon
endpoint changes nothing about the build.

**Rationale:** ADR 0080 keeps correctness in the building plane. Observation
must never become a precondition.

### Decision: Facts publish on change, not on a timer

**Choice:** Publish a fact when it becomes true and retract it when it stops
being true. No heartbeat.

**Rationale:** A clock-based liveness model makes failure a timeout. The
reviewed model ties fact lifetime to owner lifetime, so worker loss is a
retraction.

### Decision: Daemon restart retracts all facts

**Choice:** Daemon restart drops every published fact and publishes the
retraction.

**Rationale:** Coordination has no durable state. A restart means every fact
is unknown until re-emitted, and a subscriber must see that.

### Decision: Live facts are not evidence

**Choice:** The daemon authors no receipts, and its facts are rejected by
evidence validators.

**Rationale:** An assertion's meaning depends on its lifetime. Evidence must
be reproducible from stored facts.

## Risks / Trade-offs

- Two components must agree on fact identity. The building plane owns the
  identity, and the daemon validates it.
- A daemon adds a process to operate. It stays optional, with a bounded local
  fallback for development.
- Remote observability is deferred; a later change can replicate facts.

## Non-Claims

- A live fact proves current coordination belief only.
- Terminal live facts are not build receipts, and they do not prove success.
