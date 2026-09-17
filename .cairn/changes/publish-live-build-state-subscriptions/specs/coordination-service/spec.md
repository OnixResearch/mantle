# Specification: Coordination daemon

## ADDED Requirements

### Requirement: The daemon serves live-state subscriptions

r[mantle.coordination_service.live_state_subscription] The coordination daemon
MUST serve subscriptions over build facts. A subscriber MUST receive the
current matching fact set on subscription and MUST receive later changes that
match its bounded filter.

Fact size, fact count, filter size, and subscriber count MUST be bounded. An
oversized or malformed filter or fact MUST be rejected. A slow subscriber MUST
be dropped under a declared bound without affecting any build.

#### Scenario: Subscription order

- GIVEN a multi-root build and a subscriber whose filter matches two roots
- WHEN the build discovers, dispatches, and completes those roots
- THEN the subscriber MUST receive matching facts in the order the build
  emitted them
- AND unrelated roots MUST NOT appear

#### Scenario: Slow subscriber is dropped safely

- GIVEN a subscriber that stops reading
- WHEN its pending facts exceed the declared bound
- THEN the daemon MUST drop that subscriber with a bounded event
- AND no build MUST be affected

### Requirement: Facts retract when their owner stops

r[mantle.coordination_service.retraction_on_owner_stop] A published fact MUST
retract when the state it describes stops being true. Completion, failure,
cancellation, reservation release, worker loss, and daemon restart MUST produce
retractions. A subscriber MUST NOT infer staleness from a clock.

#### Scenario: Worker loss retracts its facts

- GIVEN a running build with a dispatched goal on one worker
- WHEN the worker session ends without a valid completion
- THEN the worker presence fact and the in-flight goal fact MUST retract

#### Scenario: Daemon restart retracts all facts

- GIVEN published facts for one or more builds
- WHEN the daemon restarts
- THEN every published fact MUST be retracted
- AND subscribers MUST observe the retraction before any new fact

### Requirement: The daemon holds no build authority

r[mantle.coordination_service.daemon_independence] The coordination daemon MUST
NOT mutate the store, author evidence, or gate admission, scheduling, or output
publication. Live facts MUST NOT be accepted as evidence by receipts, reports,
attestations, or release verification.

#### Scenario: Live fact rejected as evidence

- GIVEN a live fact record from the daemon
- WHEN an existing receipt, report, or release validator reads it
- THEN the validator MUST reject it

#### Scenario: Daemon cannot mutate the store

- GIVEN a running daemon and an operator store command
- WHEN the daemon is asked to change store state
- THEN it MUST refuse and the store MUST remain unchanged
