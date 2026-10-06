# Service readiness: current assertions, not build evidence

[ADR 0086](../adr/0086-use-declared-service-readiness-states.md) limits readiness to four consumers: the coordination daemon, Rust cache daemon, remote serve binding, and source-built fixed-point proof gates. `mantle doctor` is an optional *derived* observer, not a fifth readiness authority. This channel is local, volatile coordination state; it is never a signed proof, a build/store admission decision, a receipt, or a release claim.

## Reviewed source and Mantle interpretation

The inspected Synit reference `/home/brittonr/.local/share/mantle-references/synit-book/pages/12-operation__service.md:49-76` declares `depends-on` (lines 49-55), defines state as the **union of current assertions** (lines 57-64), and explicitly says `started` + `ready` are simultaneous: `ready` is asserted **in addition to** `started` (lines 68-70). `started` alone means startup has begun, not that requests can be handled. `failed` and `complete` are other built-ins; custom states require declaration in Mantle. The inspected restart reference `/home/brittonr/.local/share/mantle-references/synit-book/pages/18-operation__builtin__daemon.md:68-89` defines exactly `always`, `on-error`, `all`, `never`, with the group-level meaning of `all`. These are reviewed source paths, not a runtime dependency or a claim that Synit is running in Mantle.

## Observe a live service

For the optional local socket start/stop and limits, see [Live build-state subscriptions](live-build-state.md). If the coordination daemon is running, observe current readiness with the same subscription protocol (enter the JSON line after connecting):

```sh
socat - UNIX-CONNECT:/tmp/mantle-coordination.sock
```

```json
{"op":"subscribe","filter":{"owner":null,"kind":"service_readiness","subject_prefix":"service/"}}
```

The server first sends `reset` for this daemon generation, then current `snapshot` fact lines followed by `snapshot_end`; later changes are `publish`/`retract`. These are current facts, not a historical event log. On EOF discard the old view; after reconnect/reset discard **all** old-generation assertions and resubscribe. Do not infer readiness from a stale snapshot, a `started` fact, an absent degraded-observation warning, or doctor output. A missing daemon means **unknown observation**, not that a service necessarily failed.

A `service_readiness` fact has identity `(owner, kind, subject)`. Use `crunch_live_state_core::Fact::new` to construct its ID rather than copying a fabricated hash. A single live publisher process session owns its unique `owner`; simultaneous owners may not collide. `service/rust-cache-daemon/started` and `service/rust-cache-daemon/ready` are distinct stable subjects and therefore distinct fact IDs. To see that the service is ready, both assertions must be current in this daemon generation: a publish updating one ID never replaces the other. Session close removes both; daemon restart removes all facts and requires republication. The publisher must retract `ready` when work can no longer be handled or a required dependency is lost.

The fact's `state` field is **serialized JSON for one assertion** under `mantle-service-readiness-v1`, not a plain string or a union of states. For illustration, these are two *decoded state values*, **not complete publishable facts** and not IDs to paste into the socket:

```json
{"schema":"mantle-service-readiness-v1","service_id":"rust-cache-daemon","state":"started","custom_states":[],"dependencies":[],"restart_policy":"on-error","blocked_by":null,"coordination_state":true}
{"schema":"mantle-service-readiness-v1","service_id":"rust-cache-daemon","state":"ready","custom_states":[],"dependencies":[],"restart_policy":"on-error","blocked_by":null,"coordination_state":true}
```

The schema requires all eight fields and exactly one asserted state per fact. An assertion is at most 1,536 bytes; a service ID is at most 64 ASCII identifier characters, up to eight dependencies and eight declared custom states of at most 48 characters each. `blocked_by` is null or a declared dependency ID. Daemon ingress rejects unknown schema/state, undeclared custom state, missing/unknown policy, wrong state/subject association, contradictory declarations, and `ready` before the same service's `started` or while a dependency blocks. Empty dependency lists mean no dependencies, not omitted declarations. This state is additionally subject to the live daemon's 2,048-byte fact and 4,096-byte frame limits.

## What promotes or withdraws readiness

- **Coordination daemon:** starting its process is `started`; `ready` follows the ability to handle an actual subscription request. After restart subscribers discard the prior generation, then wait for both new assertions.
- **Rust cache daemon:** loading admission policy and checking a peer precedes serving requests. Assert `ready` only after a real admitted request succeeds; cache trust and C cache policy remain independent.
- **Remote serve binding:** a created binding is not enough. Assert `ready` after a real request succeeds; use the coordination socket, never remote stdio frames, stdout, or stderr to transport readiness.
- **Source-built fixed-point proof gates:** derive the declared predecessors from existing signed-plan output references, without changing signed plan bytes/digests. A current predecessor's `ready` or `complete` can unblock a gate; it does not activate StageX/protected proof or confer release eligibility.

If service `b` declares dependency `a`, report `blocked_by: "a"` while `a` lacks a current `ready` or `complete`; `b` may add `ready` alongside `started` only after that prerequisite and its own real work check. If `a` later retracts readiness, withdraw `b`'s `ready`, name `a` again, and require reevaluation before republishing. Termination before the first readiness is `failed`, whether exit is normal or abnormal, even if policy is `never` (Mantle's explicit precedence over the source's abnormal-exit-as-`complete` rule).

## Restart policies

Each assertion contains one required policy; ingress rejects an unknown or missing policy. The policy applies only to the named managed component/group, not arbitrary user programs:

| Policy | Normal termination | Abnormal termination |
| --- | --- | --- |
| `always` | Restart this process, not peers | Restart this process, not peers |
| `on-error` | Do not restart | Restart this process, not peers |
| `all` | Do not restart | Restart the whole named daemon group |
| `never` | Do not restart, terminal `complete` | Do not restart, terminal `complete` in Synit's matrix; Mantle instead reports `failed` if exit occurs before readiness |

For a pre-ready exit, first report `failed` rather than claiming `complete`; restart action is still selected by the declared policy. A restarting owner/session's earlier `started` and `ready` assertions must be retracted, and its new process must earn readiness again by serving work. No general process supervisor is implied.

## Doctor and missing endpoints

`mantle doctor` may publish a **transient derived** report when the daemon is available. Exact existing doctor human/JSON output bytes and evidence/receipt bytes remain unchanged; no new stdout field is a report carrier. A diagnostic fact carries `coordination_state: true`, not evidence authority. Do not use doctor reports or service facts to admit a build, C cache entry, protected proof, or release.

Absent or nonreading coordination endpoints degrade observation only. A one-shot best-effort publication is capped at 100 ms and cannot block builds or alter their outputs/receipts; optional publishing never implies delivery was confirmed. For publisher queue, subscriber backpressure, ownership, and generation details see [the live-state guide](live-build-state.md). Readiness assertions cannot repair or override a missing signed-plan predecessor or canonical evidence.
