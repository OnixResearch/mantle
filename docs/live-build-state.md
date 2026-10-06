# Live build-state subscriptions

[ADR 0088](../adr/0088-live-build-state-coordination.md) selects a bounded, local coordination daemon for ephemeral live build status. This is an **optional observation channel**, separate from the final `--json build` report and `--evaluation-stream` NDJSON. It is not an input to build admission, scheduling, output publication, PathInfo, receipts, attestations, or release verification. Do not use a live fact as evidence of a successful build.

## Start, observe, and stop

In separate terminals, from the Mantle repository root (with the relevant binaries on `PATH`):

```sh
mantle-coordination serve --socket /tmp/mantle-coordination.sock
```

```sh
socat - UNIX-CONNECT:/tmp/mantle-coordination.sock
# Type this one complete JSON line followed by Enter:
{"op":"subscribe","filter":{"owner":null,"kind":null,"subject_prefix":null}}
```

```sh
MANTLE_COORDINATION_SOCKET=/tmp/mantle-coordination.sock mantle build examples/hello.ncl
```

The daemon must be running before the build if a subscriber is to see that build's facts. `socat` is a separate Unix-socket client utility; the CLI does not spawn it. Start the daemon as a local process that can access the socket path. Subscribers are not required for builds, and a build without `MANTLE_COORDINATION_SOCKET` has no live-state dependency. Setting the variable opts into **best-effort, bounded, nonblocking** emission: an absent/stalled daemon degrades observation but must not fail a build or change its outputs/receipts. Live-state is change-driven, not a heartbeat or durable history. A disconnected subscriber misses events and must reconnect for a fresh snapshot; it cannot recover past transitions from this service.

When an opted-in publisher observes a failed endpoint it tries to send one
machine-readable JSON diagnostic to **stderr**, for example
`{"schema":"mantle-live-observation-v1","status":"degraded","reason":"endpoint_unavailable"}`.
This is degraded observation, not a failed build; it never enters the
report or receipts and is not evidence. One process-owned detached writer
serves at most one pending diagnostic; a full stderr pipe can stall that
writer, but cannot stall the build or spawn one thread per later failure.
Later notices are dropped when its single pending slot fills, while each
publisher retains its own degraded-observation flag. Absence of a diagnostic
does **not** prove delivery: the pipe may be full or the asynchronous writer
may race build-process exit.

For long-lived in-process consumers,
`crunch_build::live_observation_diagnostic_drops()` returns the saturating
process-local count of notices discarded because the writer is unavailable or
its one-slot queue is full. This count and each publisher's degraded flag are
observation state, not durable receipts or proof of any individual delivery.

The diagnostic's `schema` and `status` are fixed as shown. `reason` is a
bounded code: `empty_endpoint`, `endpoint_unavailable`, `connect_timeout`,
`socket_write_failure`, `socket_write_timeout`, `socket_read_timeout`,
`endpoint_rejected`, `endpoint_disconnected`, `end_flush_timeout`,
`publisher_queue_full_or_closed`, `frame_limit`, `serialization_failure`,
`invalid_fact`, `fact_rejected`, or `transition_limit`. These classify
observation failure only; do not parse tracing text or treat an absent line
as proof of delivery.

## Wire protocol

One UTF-8 JSON object per newline on a **local Unix socket**. Each new connection first receives a `reset` with that daemon process's generation, before handling requests:

```json
{"op":"reset","generation":"<daemon-generation>"}
```

The generation string is opaque, not a clock or proof. On the subscriber connection send a filter (explicit `null` fields select all facts):

```json
{"op":"subscribe","filter":{"owner":null,"kind":null,"subject_prefix":null}}
```

A narrower filter, for example `{"owner":"build-a","kind":"goal","subject_prefix":"root/"}`, conjunctively matches the exact owner, exact kind, and subject prefix. The daemon sends zero or more `{"op":"snapshot","fact":FACT}` records in deterministic order, then `{"op":"snapshot_end"}`. After this terminator, matching changes arrive as `{"op":"publish","fact":FACT}` or `{"op":"retract","id":"ID"}`. A changed state can publish the same stable `ID` again; remove/replace that entry in your local set. Facts outside the filter never enter that set. The frame examples below use `FACT` and `ID` as placeholders, **not valid JSON to send**:

```text
{"op":"snapshot","fact":FACT}
{"op":"snapshot_end"}
{"op":"publish","fact":FACT}
{"op":"retract","id":"ID"}
```

A publisher connection sends `{"op":"publish","fact":FACT}` or
`{"op":"retract","id":"ID"}` as NDJSON. Successful publisher operations return
`{"op":"ack"}`; rejected commands return `{"op":"error","message":"..."}`
(the message is diagnostic, not a stable machine API). A frame exceeding the
maximum may close the connection instead of returning an error. Produce facts
with the `crunch-live-state-core` `Fact::new` API, not a guessed ID.

The 4,096-byte frame limit includes the terminating newline: an otherwise
valid request exactly at that limit is accepted, while a 4,097-byte request
is rejected. A blocking publisher uses one two-second monotonic deadline for
connecting and one for each complete response, including all partial reads;
an endpoint sending a slow trickle cannot extend a response's deadline.

A fact is JSON with `version` (currently `1`), `id`, `owner`, `kind`, `subject`,
and `state` fields; its `kind` is one of `goal`, `worker`, `reservation`,
`outcome`, `service_readiness`. `id` is domain-separated BLAKE3 of
`(owner, kind, subject)` and does **not** include `state`. The producer
determines identity; the daemon checks it. The `owner` field is descriptive
and filterable, but connection lifetime, not this string, owns publication
rights. There is no ambient store of facts.

The daemon permits one active publisher connection per `owner`; use a distinct
service/process-session owner for each concurrent producer. Reusing an owner
for another producer is rejected rather than transferring its facts.

For `service_readiness`, `state` is **not** a free-form word: it is the
serialized `crunch-service-readiness-core::ServiceAssertion` with schema
`mantle-service-readiness-v1`, a declared service, state, dependencies,
restart policy, blocker (if any), and `coordination_state: true`. Its
`subject` must equal the assertion's `service/{service_id}/{state}` identity.
Ingress rejects raw `"ready"`, unknown states, missing or unknown restart
policies, and ready assertions without an active same-owner started assertion
or with blocked dependencies. A service's started and ready assertions have
**distinct fact IDs** and remain present together after a real accepted
request; ready does not overwrite started. The daemon publishes its own
started assertion after binding, but not ready until it accepts a subscriber
request. Restart invalidates both old assertions before a new generation can
announce readiness.

| Limit | Maximum |
| --- | ---: |
| Serialized fact | 2,048 bytes |
| Concurrent facts | 4,096 |
| Serialized filter | 256 bytes |
| Subscribers | 128 |
| Pending events per subscriber | 64 |
| NDJSON frame | 4,096 bytes |
| Aggregate concurrent snapshot capture | 64 MiB |
| Single-frame subscriber write stall | 5 seconds |
| Diagnostic writer threads per process | 1 |
| Pending degraded diagnostics per process | 1 |

Malformed or over-limit facts and filters are rejected. Atomic snapshot
captures share a 64 MiB aggregate byte budget; a subscription may be rejected
when many large snapshots are in progress. Snapshot facts are written one by
one before `snapshot_end`. A subscriber that cannot keep up with subsequent
changes is dropped when its 64-event queue fills, or when a single frame write
stalls for more than five seconds, without stalling a build. The daemon does
not provide a remote endpoint, event replay, credential transport, or
store/evidence authority. Protect access to the local socket through your
operating environment; do not place secrets in descriptive fact fields.

## Publisher APIs and wait boundary

`crunch_coordination::BlockingPublisherSession::connect(socket)` keeps one
publisher connection open and waits synchronously for acknowledgments on
`publish(&fact)` and `retract(id)`. Reserve this for explicit doctor/diagnostic
operations; do not put it on build scheduling or remote/cache critical paths.
For synchronous service code use `BestEffortPublisher::new(socket)`, then
`try_publish(fact)` / `try_retract(id)` to enqueue onto one background thread
without waiting for the daemon. Its queue holds at most 64 commands; an
unavailable socket, rejected command, or full queue degrades observation.
`finish(deadline)` waits at most 100 ms and returns whether queued work
drained. A batch build uses a separate async publisher whose final flush is
also capped at 100 ms outside scheduler dispatch; the limit protects build
latency but is not a delivery receipt.

Those bounded APIs and daemon tests alone do not establish identical actual
build outputs and receipts with emission enabled versus disabled. That parity
requires an exercised CLI build comparison.

## Ownership and restart

A publisher **connection** owns all facts it sends. Explicit retract (for example, a completed goal or released reservation) removes the ID from current sets. Connection loss retracts all facts sent on that connection and connected subscribers receive per-ID `retract` events. If a worker dies while its publisher remains connected, the publisher must observe the loss and retract worker-presence and affected in-flight goal facts; the daemon cannot infer that worker's death from the `owner` text. Terminal outcomes are volatile observations, not successful-build receipts.

Restart clears all in-memory facts. The old process cannot send individual
retractions after it dies. For a raw `socat` or other NDJSON client, close your
prior view on EOF, discard **all** prior-generation facts when a new `reset`
arrives, and send `subscribe` again for the new generation's snapshot.

For a programmatic client, `crunch_coordination::SubscriptionClient::new()`
tracks `current()` facts and `generation()`; pass each complete frame's bytes
to `receive(&[u8])` and call `disconnected()` on EOF. Both `disconnected()`
and a newly received `reset` produce sorted `SubscriptionEvent::Retract { id }`
events for **every retained ID before** the next
`SubscriptionEvent::Reset` and snapshot. These explicit restart retractions
are synthesized by the client, not delivered by the dead daemon. No old
publication is replayed; publishers must reconnect and republish true
current facts.

Within one generation process `snapshot` through `snapshot_end` before
treating `publish`/`retract` as updates. A dropped slow subscriber reconnects
and rebuilds its local set from a new reset and snapshot.

## Provider and limits of the claim

The local Aspen Molten source implements exact/whole-value-wildcard
observations, current matching assertions, and explicit retractions. The
reviewed API and tests do not provide Mantle's bounded filter, Unix socket,
subscriber backlog, or connection-lifetime reset protocol; this daemon uses a
Mantle-owned pure fact set rather than importing Molten. Neither that provider
decision nor these examples prove Molten/Mantle interoperability or build
identity. Separately, an actual primary-source CLI snapshot has passed
two-root enabled/disabled output-and-entire-receipt comparison, including a
full stderr pipe: see
`.cairn/changes/publish-live-build-state-subscriptions/evidence/live-build-verification.txt`.
That snapshot is not a final integrated-branch proof. ADR 0088 records the
source/API review and its precise non-claims.
