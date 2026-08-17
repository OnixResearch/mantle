# Remote attempt log inventory and migration treatment

Date: 2026-07-12

Change: `persist-remote-attempt-observability`

Requirement: `r[remote_builds.immutable_attempt_log_segments]`

## Question

Where do remote attempt log append, replay, retention, and persistence live before this change, and how may legacy mutable logs cross the future immutable-log boundary without fabricating provenance?

## Inspected evidence

- `crates/crunch-build/src/distributed/remote_attempt.rs` owns pure durable job/attempt/fence identity, event idempotency, attempt transitions, retry, and stale-fence rejection.
- `src/remote_build.rs::RemoteCoordinatorState::logs` stores mutable `BTreeMap<RemoteJobId, Vec<RemoteCoordinatorLogChunk>>` values inside the main coordinator snapshot.
- `src/remote_build.rs::apply_fenced_log_append()` clones the current vector, calls `retain_remote_log_chunks()`, then replaces the vector in candidate coordinator state.
- `src/remote_build.rs::retain_remote_log_chunks()` validates monotonic cursors, appends one mutable chunk, trims from the front by count/bytes, and derives replay from the remaining vector.
- `src/remote_build.rs::save_coordinator_state()` persists the whole coordinator state snapshot; immutable segment files and a separately advanced manifest do not yet exist.
- Legacy `RemoteCoordinatorLogChunk` values may omit `event_id` and `payload_digest`; they do not bind attempt id, fence generation, previous-record digest, record digest, segment digest, policy identity, or an explicit truncation anchor.
- `crates/crunch-build/src/distributed/remote_transfer.rs` and `src/remote_transfer.rs` own a separate fenced resumable-transfer core/shell. The active `complete-resumable-remote-cas-transfer` package still leaves production remote client/server integration incomplete, so transfer telemetry is not evidence for this log slice.

## Decision

Legacy mutable vectors MUST NOT be rehashed and described as historical immutable attempt-log segments. They lack the facts required to prove that identity.

This partial slice adds an independent pure immutable-log core only. Until a shell migration lands:

1. Existing coordinator vectors remain legacy diagnostic state and are not promoted into the new manifest chain.
2. A future shell may start immutable logging only for a newly assigned fenced attempt whose first record can bind a complete job/attempt/fence scope.
3. Legacy chunks must remain explicitly classified as legacy/unbound diagnostics until existing bounded retention removes them; they cannot become a truncation anchor because no dropped-tail record or segment digest existed.
4. Coordinator vector replacement, immutable file publication, atomic manifest advancement, restart migration, and deletion ordering remain unchecked work in this change.

## Owner

The active Cairn change `persist-remote-attempt-observability` owns immutable-log shell integration and any explicit legacy-boundary schema needed later.

## Next action

Add a thin shell that persists a content-addressed segment, fsyncs it, atomically advances a bounded per-attempt manifest, and only then updates bounded coordinator summaries. Prove restart, tamper, and retention deletion ordering before removing the legacy vectors.
