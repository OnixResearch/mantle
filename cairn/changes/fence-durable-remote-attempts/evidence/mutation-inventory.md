# Remote attempt mutation inventory

Change: `fence-durable-remote-attempts`
Task: Implementation inventory
Date: 2026-07-12

## Pre-change findings

| Mutation surface | Pre-change identity | Missing safety facts | Consequence |
|---|---|---|---|
| Coordinator dispatch admission | normalized realization key plus string job id | attempt id, fence generation | A replacement owner could not be distinguished from the original assignment. |
| Dispatch persistence | same logical job fields | durable pre-exposure commit result | In-memory jobs and output claims were mutated first; save failures were reduced to warnings and the dispatch was still returned. |
| Worker resume adoption | job id plus normalized key | attempt id, fence, durable-owner comparison, event identity | A reconnecting worker could overwrite assigned worker, phase, result availability, and log cursor, or create an unknown job from worker claims. |
| Start/heartbeat/state changes | request/job-shaped protocol facts | attempt id, fence, event id, canonical payload digest | Late state reports could not be classified as current, duplicate, conflicting, or stale. |
| Log append/retention | monotonically increasing cursor | job/attempt/fence/event/digest authorization | Cursor monotonicity bounded storage but did not prove that the appending worker still owned the job. |
| Transfer checkpoints | request id and transfer counters | current attempt/fence plus idempotency identity | A superseded worker could advance or replace transfer progress. |
| Result-ready/output admission | request id, output digests, signing trust | current attempt/fence and stable event identity | Cryptographic output trust was separate, but current assignment ownership was not checked before admission. |
| Failure/completion | logical request/job state | current attempt/fence and event conflict detection | Late failure or completion could race a replacement attempt under the same logical job. |
| Durable reload | serialized job summary | safe migration for missing attempt/fence state | Legacy live or finished-undelivered records could be interpreted without a provable current owner. |
| Retry/reconnect policy | raw attempt counters and ad hoc time arguments | named bounded policy and explicit immutable time facts | Retry decisions were not represented in typed Nickel farm policy or one reusable deterministic kernel. |
| Status/build reports | phase, worker, bounded error text | typed attempt/fence and stable fencing reason codes | Operators could not distinguish supersession, stale rejection, or current completion. |

## Implemented ownership boundary

- `crates/crunch-build/src/distributed/remote_attempt.rs` owns typed identities, canonical BLAKE3 payload digests, bounded retained event bindings, fence comparison, authorization, transitions, retry decisions, and report application plans.
- `src/remote_build.rs` owns worker transport authorization facts, candidate-state mutation, atomic durable writes, log storage, transfer/result fields, output-admission plumbing, migration, and rendering.
- `src/remote_farm_config.rs` and `lib/remote-builders.ncl` own provider-neutral typed retry policy; callers supply time facts explicitly.
- The coordinator clones candidate state, persists it, and only then replaces visible in-memory state or returns a new assignment.
- Worker resume summaries can confirm only the coordinator's existing durable owner; they cannot create or overwrite coordinator jobs.
