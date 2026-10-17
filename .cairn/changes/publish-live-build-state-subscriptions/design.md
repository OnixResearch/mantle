# Design: Live build state for coordination subscribers

## Goal and scope

One coordination daemon serves current build facts and retracts them when they
stop being true. The building plane emits facts and keeps its determinism and
its independence from the daemon.

## Current behavior and isolated baseline

An isolated detached `origin/main` worktree at
`da00f58425740adea559ef926c9dfa97b2cb8240` had aggregate
`crunch-build-report-v1` and explicit evaluation NDJSON, but no subscription
CLI or subscriber retractions. Its worker-loss ledger and remote
resource-release tests recorded terminal/capacity transitions, not live
facts. Actual CLI and focused-test commands/results, including the
preexisting bwrap-gated test skip, are in `evidence/baseline-process.txt`.
The baseline did not treat a nominal skipped test as build-path proof.

## Approach review

| Family | Mechanism | Disposition | Required check |
| --- | --- | --- | --- |
| Extend the NDJSON stream | Add subscribe/snapshot records | Rejected: a stream is not a state view; retraction stays implicit | Worker-loss fixture |
| Poll the aggregate report | Repeated `--json` runs | Rejected: expensive and stale between polls | Subscription-order fixture |
| Fold subscriptions into the build process | Long-lived build serving consumers | Rejected by ADR 0080: consumers would depend on a resident build | Daemon-absence fixture |
| Coordination daemon over Molten dataspaces | Build emits facts; daemon serves and retracts | Selected direction | Molten contract review plus all fixtures |

### Molten provider review (T1.3)

The locally reviewed Molten runtime has `Observe`, `Assert`, and `Retract`
steps; an `Observe` delivers matching current assertions and later assertions,
and a `Retract` notifies observers
(`aspen/src/runtime/dataspace/parts/state/p001/body.rs:181-240`, verified
by `aspen/src/runtime/dataspace/parts/tests/p000/body.rs:77-106`).
Its actor-scope cleanup retracts owned assertions
(`aspen/src/runtime/dataspace/parts/syndicate/p000/body.rs:153-186`).
The currently exposed `RuntimePattern` admits only exact values and one
wildcard binding, with at most 128 bytes in that binding
(`aspen/src/runtime/predicates/parts/mod/p000/body.rs:70-160`).
The local adapter stores subscriptions by actor and routes matches
(`aspen/src/runtime/dataspace/mod.rs:122-179`); the reference harness
has a configurable fanout budget, not Mantle's per-filter fact/count,
subscriber-count, and slow-consumer queue limits.

**Decision:** Molten supplies a possible assertion/retraction routing primitive,
not a reviewed end-to-end Mantle subscription service. The local implementation
therefore uses a narrow Unix-socket daemon port with bounded Mantle filters,
snapshot ordering, subscriber quotas/drop, and restart cleanup. A later Molten
binding must prove the same behaviours before replacing this port. Do not
replace it with a second actor runtime or promote the daemon to authority.

## Contract and component ownership

- Building plane (pure core): fact normalization, fact identity, bounded fact
  and event admission, and the mapping from scheduler state to emit actions.
- Building plane (shell): emission to the daemon or to a bounded local
  channel. Emission failure is non-fatal.
- Coordination daemon (pure core): subscription filter validation, current
  matching set, and retraction decisions over in-memory facts.
- Coordination daemon (shell): surface lifecycle, subscriber handling, and
  rendering. The daemon never blocks a build on a subscriber.
- Provider: Molten supplies the local observation/retraction primitive;
  a narrow daemon-owned port must enforce Mantle's missing filter and lifecycle
  rules before it can serve subscribers.

### Pure building-plane fact vocabulary (T1.2/T2.1)

`src/remote_build/live_state.rs` projects remote coordinator snapshots into
versioned `mantle-live-build-fact-v1` facts. An explicit set of *present worker
sessions* is an input: a durable registration alone does not establish worker
liveness. The building plane hashes the owner run id plus a domain-separated,
length-prefixed fact kind and its stable identifiers with BLAKE3:

The evaluation-stream `run_id` remains content-addressed and unchanged in
NDJSON and reports. The imperative live publisher assigns a separate
per-invocation owner id from that run id, process id, and checked in-process
sequence. Two concurrent builds of identical inputs therefore retain distinct
daemon owners and fact identities; stopping one cannot retract the other.

| Fact | Identity and value | Lifetime in the pure projection |
| --- | --- | --- |
| Worker presence | Endpoint id and incarnation generation | Remote: only an explicitly present session, never registration. Local: only the observed in-process Worker invocation; its owner-scoped endpoint is withdrawn with the build owner. |
| Goal | Job id; optional remote attempt and fence; value is `discovered` or `dispatched` | Remote running jobs require a present worker, attempt and fence. A real local Worker scheduler dispatch has a local endpoint and no invented remote attempt/fence. A phase change retracts the previous value first. |
| Reservation | Resource lease id plus job id; value includes assigned worker, attempt, fence | Only a real current coordinator resource lease with a present assigned worker is published. Local scheduler slots are not fabricated as leases. |
| Terminal outcome | Job id plus optional attempt id | Remote `finished`/`lost`, selected-root evaluation outcomes, and actual local Worker `succeeded`/`failed` remain until owner stop. None is output-trust or receipt evidence. |

The remote coordinator projection admits at most **4,096** simultaneous facts,
**2,048** serialized bytes per `{owner_run_id, fact_id, fact}` envelope,
and **256** UTF-8 bytes per identity field. A diff emits at most **8,192**
actions, with retractions before publications; unchanged facts do not republish.
Invalid identities, mismatched leases, unknown worker sessions, and excess
facts fail this *observation* normalization. They never reject a build.
`plan_remote_live_owner_stop` derives retractions but cannot deliver them.

The opt-in building shell reads `MANTLE_LIVE_STATE_SOCKET` and uses the
nonblocking, bounded `BestEffortLivePublisher` worker, which owns the Unix
socket. `src/build_cmd.rs` projects evaluation-stream records in ordinary and
`--evaluation-stream` builds: selected-root discovery becomes `discovered`,
and each root terminal retracts its discovery before publishing status.
Separately, `crunch-build::Worker` exposes the **actual** post-discovery,
post-dispatch and post-completion goal registry to the pipeline's optional
bounded, nonblocking observation channel. Local Worker goals project to
`discovered`/`dispatched`/terminal facts with an owner-scoped in-process
worker presence, **without** invented remote attempts, fences or resource
leases. The selected-root stream is not used to infer scheduler dispatch
or output admission. Exceeding the 4,096-fact/4,095-Worker-goal observation
bound or filling the optional worker channel degrades only observation.

The remote `run_remote_build_dispatches_async` shell observes coordinator
state after each accepted dispatch and after attempt completion/failure. It
passes only the jobs admitted by this invocation to
`normalize_remote_live_facts_for_jobs`, excluding old persisted terminal
jobs. A configured worker registration does **not** establish a live session;
the current one-shot stdio shell therefore passes an empty presence set and
does not fabricate worker/reservation/dispatched facts. Its real accepted
queued and terminal job transitions are still observable. A failed endpoint
or fact admission logs an observation diagnostic but cannot alter the build
outcome. Wire delivery is best effort without publisher acknowledgements.

`src/remote_build/live_state/daemon.rs` owns a separate Unix-socket server
(`remote live serve`), never a store handle. It admits at most 4,096 facts,
4,096-byte frames, 64 filter entries, 32 subscribers, 128 connections, and
a 64-event subscriber queue; slow subscribers are disconnected rather than
blocking publishers. A subscription (`remote live subscribe`) receives
`snapshot-start`, matching facts, `snapshot-end`, and then publish/retract
events. Build-owner disconnect withdraws its facts. A crashed daemon cannot
deliver retractions; subscribers invalidate their former snapshots on
disconnect, and a restarted daemon begins with an empty state.

Graceful shutdown closes the live fact set and queues each owner's retractions
for subscriber writers before waiting for the separate readiness reporter.
Only a previously owned readiness component may acknowledge its unchanged
terminal exit during that drain; new starts, ready claims, restarts, and foreign
updates remain rejected. The daemon stops client handlers after that terminal
readiness exchange, so a slow readiness probe cannot withhold live retractions.

### Process and scoped-test evidence

The separate command/output transcript is in
`evidence/live-cli-process.txt`, with focused test commands and the historical
versus current proof boundary in `evidence/focused-tests.txt`. The actual
`tests/fixtures/multi.ncl` no-endpoint, missing-endpoint, refused-endpoint
and managed Unix-daemon runs exited successfully; no-endpoint and
managed-endpoint outputs and raw receipts were byte-identical. The managed
subscriber saw snapshot bracketing, selected-root publications and
retractions; owner disconnect cleared them. The CLI rejected invalid
filters, the receipt parser rejected a genuine live fact, raw store-mutate
frames were refused without changing the store database, and restart began
with an empty snapshot. A crashed daemon cannot send per-fact retractions;
disconnect invalidates its old snapshot. That earlier transcript predates the
actual local Worker scheduler hook. A separate SHA-pinned first integrated
binary process run used the same two-root Nickel source in two truly
simultaneous CLI invocations. The subscriber observed each actual `.drv`
goal/dispatched fact followed by build terminal facts and retractions, with
distinct per-invocation owners and worker-presence endpoints; the NDJSON run id
remained the same content identity. A fresh snapshot held both owners; after
one build exited it held only the other; after both exited it was empty.
No-endpoint and concurrent live builds produced byte-identical physical
outputs and raw signed artifact receipts. Exact binary/source hashes, socket
frame counts, selected derivations, and process output are retained in
`evidence/live-cli-process.txt`; a newer binary after the remote/watch/S0
cutovers belongs to the second integration gate.

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

### Decision: Daemon restart invalidates all old facts

**Choice:** A graceful stop retracts published facts before dropping
subscribers. After a crash or restart, previous connections are invalid and
the next subscription begins with an empty current set until owners publish
again. A subscriber treats the disconnect as invalidating all prior facts;
the dead process cannot promise delivery of individual retraction messages.

**Rationale:** Coordination has no durable state. Restart must never
resurrect an assertion from the previous daemon lifetime.

### Decision: Live facts are not evidence

**Choice:** The daemon authors no receipts, and its facts are rejected by
evidence validators.

**Rationale:** An assertion's meaning depends on its lifetime. Evidence must
be reproducible from stored facts.

## Risks / Trade-offs

- Two components must agree on fact identity. The building plane owns the
  identity, and the daemon validates it.
- A daemon adds a process to operate. It stays optional; a missing socket
  degrades observation instead of activating an in-process substitute.
- Cross-machine replication is deferred; the implemented socket is local.

## Non-Claims

- A live fact proves current coordination belief only.
- Terminal live facts are not build receipts, and they do not prove success.
