# Proposal: Publish live build state to coordination subscribers

## Why

Consumers observe a running build through a one-shot NDJSON stream or a final
JSON report. Onix lowering, CI, and operator tools must poll or parse a stream
to learn what is running now, and a dead worker leaves no way to retract a
stale "building" fact.

The Synit manual routes published state through dataspaces: peers declare
interest with a pattern, receive matching assertions, and receive a retraction
when the publisher withdraws or dies
(`~/.local/share/mantle-references/synit-book/pages/07-syndicated-actor-model.md`,
reviewed in `docs/synit-application-notes.md`). A build has exactly these
facts: goals, worker presence, reservations, and terminal outcomes.

ADR 0080 separates the coordination daemon from the building pipeline. The
build therefore emits bounded facts and stays batch-shaped. The daemon owns
fact lifetime and subscription service. The stack already owns a dataspace
component: Aspen's Molten runtime provides dataspaces, Preserves identity, and
coordination delivery. The daemon consumes that component instead of defining
a second coordination model.

## What Changes

- Emit bounded, versioned build facts and events from the building plane:
  goal state, worker presence, reservations, and terminal outcomes. Emission
  MUST NOT change build behavior, and a build MUST run unchanged with no
  daemon reachable.
  r[mantle.build_interchange.live_state_emission]
- Serve subscriptions from the coordination daemon. A subscriber MUST receive
  the current matching fact set on subscription and later changes that match
  its bounded filter.
  r[mantle.coordination_service.live_state_subscription]
- Retract facts while the daemon runs when their owner or asserted state stops:
  a completed goal, a released reservation, a lost worker, or a cancelled root.
  Graceful daemon shutdown retracts before disconnect. A crash cannot emit
  frames; subscribers MUST invalidate prior facts on disconnect, and restart
  begins empty rather than reviving assertions.
  r[mantle.coordination_service.retraction_on_owner_stop]
- Keep the daemon outside build authority. It MUST NOT mutate the store,
  author evidence, or gate admission, scheduling, or output publication. Live
  facts MUST NOT be accepted as evidence.
  r[mantle.coordination_service.daemon_independence]
- Evaluate the Molten dataspace contract first. If it cannot carry the
  required bounded filters and retraction, record the decision and keep the
  surface behind a narrow daemon-owned port.
- Keep `--json build` and `mantle-evaluation-stream-v1` unchanged.

## Impact

- **Immediate consumer**: operator tooling and CI viewers that currently parse
  the evaluation stream for live status.
- **Immediate outcome**: status is subscribed instead of polled, and worker
  loss produces an explicit retraction.
- **Durable capability**: one coordination service that later carries declared
  worker demand, readiness, and resolution.
- **Maintenance owner**: Mantle coordination daemon owner for the service, and
  Mantle scheduling owner for fact emission.
- **Repeatability evidence**: subscription-order fixtures, retraction
  fixtures for completion, cancellation, worker loss, and daemon restart, plus
  build-identity fixtures with and without a daemon.
- **Compatibility**: existing contracts stay. The surface requires an explicit
  opt-in, and the daemon is optional for every build.

## Scope

The change covers the emitted fact vocabulary, the daemon subscription
service, retraction semantics, the Molten evaluation, and the fixtures.

## Out of Scope

- Replacing the aggregate build report or the NDJSON stream.
- Remote subscriptions across hosts.
- Build admission, scheduling, or store mutation inside the daemon.
- Treating live facts as evidence, receipts, or release inputs.

## Success Criteria

- A subscriber receives discovery, dispatch, and terminal facts in order.
- Worker loss and daemon restart retract the affected facts.
- A killed subscriber does not affect the build.
- A build with no daemon produces identical receipts to a build with one.
