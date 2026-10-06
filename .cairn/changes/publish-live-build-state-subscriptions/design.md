# Design: Live build state for coordination subscribers

## Goal and existing surface

Serve a current, filtered fact set and later changes without changing batch
build correctness, the aggregate `--json build` report, or the existing
`--evaluation-stream` NDJSON. The latter is an event stream, not a current set:
it cannot retract a stale fact from a subscriber's view on worker loss.

[ADR 0088](../../../adr/0088-live-build-state-coordination.md) accepts the
narrow daemon-owned provider for this local live-state surface. The change
proposal invokes ADR 0080; its file was absent in the isolated baseline at
review time, although it is present in the original Mantle source and was
restored to this isolated branch separately. Its Decision specifies three
planes and mandates Molten use for coordination; ADR 0088 preserves the planes
but explicitly supersedes only that provider mandate for this surface. It
does not imply native Molten compatibility.

## Provider review and boundary

| Alternative | Decision |
| --- | --- |
| Add subscribe/snapshot to one-shot build NDJSON | Reject: no independent resident current set or connection-owned cleanup. |
| Poll the aggregate report | Reject: final-only, cannot report a live change/retraction. |
| Run subscriptions inside a build | Reject: readers depend on the lifetime of a batch build. |
| Directly embed Molten dataspaces | Defer: source supports observe/assert/retract but not this entire bounded socket contract. |
| Narrow Mantle daemon over pure `crunch-live-state-core` | Select for the local-only opt-in service; no substitute store/evidence authority. |

Evidence in the reviewed local tree `/home/brittonr/git/OnixResearch/aspen`
at revision `81baf80cb667a22277be22c140d34829e5c7da7f` (the path is review
provenance, not a runtime dependency or provider configuration):

- `src/runtime/dataspace/mod.rs:54-78,106-190` gives a snapshot of in-memory
  assertions/observers and `LocalAdapter::observe_pattern` routing envelope
  subjects to actors. That adapter does not serve a Unix-socket snapshot
  stream or backpressure-bound subscribers.
- `src/runtime/predicates/parts/mod/p000/body.rs:70-74,115-128,148-163`
  limits `RuntimePattern` to exact or whole-value wildcard matching. A
  conjunctive `owner`/`kind`/`subject_prefix` filter needs Mantle logic.
- `src/runtime/dataspace/parts/state/p001/body.rs:181-240` constructs
  initial matching observations and future explicit assertion/retraction
  observations. `:115-139` cleans up an actor's scope and returns references,
  but does not emit retraction observations on cleanup by itself.
  `src/runtime/dataspace/parts/tests/p000/body.rs:76-106,220-225` checks
  explicit retractions and cleanup separately, not socket disconnect/restart.
- `aspen/Cargo.toml:13-17,52-101` declares `molten` 0.1.0,
  AGPL-3.0-or-later, Preserves 5.0.0-rc.7, Syndicate 0.44.0-rc.3,
  and independent git dependencies. Importing it requires an explicit
  dependency/license review and an executable integration, not a name match.

The provider seam is the daemon-owned local publish/retract/subscribe protocol
and its bounded `LiveSet`, **not** a promised interchangeable Molten adapter.
Only an executable integration proving exact filters, ownership, socket
availability, initial delivery, backpressure and restart reset could change
that decision. No such integration is claimed here.

## Fact and filter contract

`crunch-live-state-core` is `#![no_std]` with alloc. Version `1` fact:
`{version:u32,id:String,owner:String,kind:FactKind,subject:String,state:String}`.
Kinds serialize as `goal`, `worker`, `reservation`, `outcome`, and
`service_readiness`. A filter has optional exact `owner`, exact `kind` and
`subject_prefix` (all absent means all facts). `fact_id(owner,kind,subject)`
uses domain-separated BLAKE3; state is **not** part of identity. The producer
calculates the ID and the daemon validates it. A changed state replaces the
same identity; removing a state retracts that ID. Snapshot matching and
ordering are deterministic.

Hard bounds: 2,048 serialized bytes per fact, 4,096 retained facts,
256 serialized bytes per filter, 128 subscribers, 64 pending delta events for
each subscriber, and 4,096 bytes per wire frame. Concurrent atomic snapshot
captures share a 64 MiB byte cap (`8 * MAX_FACTS * MAX_FACT_BYTES`) and are
rejected if it is exhausted; snapshot facts stream individually before
`snapshot_end`. Reject invalid IDs, version, malformed/oversized facts and
filters or exhausted fact capacity. A full subscriber delta queue or a frame
write stalled more than five seconds drops the subscriber, never a build.

Pure-core JSON byte accounting uses checked additions. Arithmetic overflow is
treated as over-limit rather than wrapping into an admitted fact or filter;
`Fact::new` checks the full serialized size before allocating owned fields.
The exact maximum serialized fact/filter sizes are accepted, and a one-byte
excess is rejected.

The **publisher socket connection** owns every fact published over it, regardless
of the `owner` field; on loss of that connection the daemon retracts its facts.
Worker loss while the publisher remains connected must be observed by the
producer and explicitly retract the worker and affected in-flight goal facts.
Do not mistake a descriptive owner field for a liveness oracle.

## Shell, lifecycle and wire

The daemon `mantle-coordination serve --socket PATH` listens on a local Unix
socket; it has no store capability and writes no evidence or receipts.
`MANTLE_COORDINATION_SOCKET=PATH mantle build ...` opts a build into bounded
best-effort, nonblocking, change-driven publication. Missing/failed/slow
daemon publication degrades observation only, never admission, scheduling,
output identity, or evidence; no socket setting is needed for a normal build.
No heartbeat and no ambient persisted fact store.
An observed publisher failure attempts one deduplicated JSON event on build
stderr, for example
`{"schema":"mantle-live-observation-v1","status":"degraded","reason":"endpoint_unavailable"}`.
One process-owned diagnostic writer has one pending slot; a permanently full
stderr pipe can stall that worker but cannot stall builds or create a thread
per later failure. Discarded notices increment the saturating process-local
`crunch_build::live_observation_diagnostic_drops()` count. This observation
state is not part of build reports or receipts and cannot affect decisions.
Missing stderr output is not proof of daemon success or diagnostic delivery.

For doctor/diagnostic calls only, `crunch_coordination::BlockingPublisherSession`
opens one synchronous session and waits for each ACK; never use it in a
scheduling, remote-service, or cache critical path. The synchronous
`BestEffortPublisher::new(socket)` places the socket session on one background
thread; `try_publish(Fact)` and `try_retract(id)` enqueue without waiting for
the daemon and degrade observation when the 64-command queue fills or fails.
Its `finish(deadline)` caps the caller's final wait at 100 ms. The build's
Tokio publisher separately caps its final flush to 100 ms after scheduling.
These bounds do not establish build-output or receipt parity until an actual
CLI run compares emission with daemon present and absent.

Each connection first receives `{"op":"reset","generation":"<daemon-id>"}`.
Requests are newline-delimited JSON:

```text
{"op":"subscribe","filter":{"owner":null,"kind":null,"subject_prefix":null}}
{"op":"publish","fact":FACT}
{"op":"retract","id":"ID"}
```

`FACT` and `ID` above are placeholders. Subscribe returns matching
`{"op":"snapshot","fact":FACT}` entries in deterministic order, then
`{"op":"snapshot_end"}`, followed by change-driven
`{"op":"publish","fact":FACT}` and `{"op":"retract","id":"ID"}`.
Successful publisher operations may return `{"op":"ack"}`; bad input returns
an `error` with a string message. See `docs/live-build-state.md` for an
operator example.

On daemon restart the in-memory set disappears. The dead daemon cannot send
per-ID retractions to sockets it already lost. Raw-wire clients discard their
old set on disconnect or new-generation `reset` and subscribe again. The
stateful `crunch_coordination::SubscriptionClient` tracks that set:
`disconnected()` on EOF returns sorted per-ID `SubscriptionEvent::Retract`s;
`receive()` on a new `reset` retracts every retained ID before returning
`SubscriptionEvent::Reset`. This client-side conversion yields explicit
retractions before a new snapshot, not events emitted by the dead daemon.
Prior publishers must reconnect and republish; no facts are replayed from
disk. Within one generation explicit removal and publisher loss deliver
per-ID retraction on the wire to live subscribers. A `service_readiness`
fact's state must parse as a versioned `ServiceAssertion` from
`crunch-service-readiness-core`; admission rejects unknown/missing policy,
unknown state, malformed subject, fabricated ready without same-owner active
started, and unmet dependencies. The daemon's own started fact precedes any
accepted request; ready is a distinct concurrently valid fact only after a
real accepted subscription. Neither fact is authority to prove service health
or release evidence.

## Limits and evidence

Live facts represent a volatile coordination belief only. They must not be
consumed as PathInfo, report, attestation, receipt or release evidence.
Fixtures and end-to-end checks must separately demonstrate ordering,
owner-loss/restart invalidation, bounded filtering/backpressure, worker-loss
retraction, and build identity with and without the daemon; this design and
the Aspen source review do **not** claim those proofs have passed.
