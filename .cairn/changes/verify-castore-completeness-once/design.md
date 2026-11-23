# Design: verify each castore node's completeness once per store handle

## Context

`recursive_castore_completeness` is deliberately a pure read of the active services: it walks every
child directory and reads plus hashes every blob. Route v2 calls it three times over the same
closure because the hydration walk, the presence predicate for non-root unit outputs, and the
build's per-input cache probe all take the same `StoreHandle`. Content identities are immutable
Merkle facts, so re-verifying the same finalized node identity within one process invocation
produces no new information.

The existing spec scenario "immutable completeness marker permits cheap repeat hit" already permits
a keyed marker. This change implements the smallest correct form of that marker: one in-memory set
on the handle, no persistence and no schema.

## Decisions

### Decision: One per-handle set keyed by finalized node identity

**Choice:** key the set by node kind (file or directory), digest, and declared size. Symlinks are
inline and return without an entry. Store the set directly in `StoreHandle` and expose one
`&mut self` method.

**Rationale:** the handle is the lifetime that spans the three route-v2 passes; a set on the handle
holds exactly the answers observed against the services this handle opened. The key is the full
finalized identity, so a different size or digest is always a different entry. Executable bits are
excluded because completeness of a file is a property of its content digest and declared size. A
`&mut self` method makes invalidation explicit and keeps the set out of shared read paths.

**Rejected alternatives:** a process-global cache (outlives the handle, admits stale answers across
store opens and state directory switches); a persisted on-disk marker (schema, invalidation, and
store-prefix facts, all beyond this follow-up); indexing by store path (a path can be reused for a
new identity and the node is the immutable fact).

### Decision: Record only success, never evict, and keep the uncached check

**Choice:** record an identity only after `recursive_castore_completeness` returns `true`. At
`MAX_VERIFIED_COMPLETE_NODES`, skip insertion and continue full checks. Leave
`castore_has_complete_content(&self)` and its free function uncached for every other caller.

**Rationale:** an incomplete or absent node must be re-checked on every call, because the next call
is exactly when the missing content may have arrived. Evicting at the bound would silently trade a
correct answer for capacity; skipping insertion only costs a repeat hash. Action-result admission is
a separate trust boundary and keeps its fresh check, as does repair inspection.

### Decision: Clear on every content-deleting handle operation

**Choice:** clear the set at the start of `garbage_collect_inner` and of
`recover_casita_gc_under_guard`. These are the only handle operations that delete castore blobs or
directories; `export_output_if_needed` only replaces a physical export, and ingestion only adds
content.

**Rationale:** clearing before the deletion guarantees no verification result survives a removal,
including a partially failed GC. Clearing before rather than after also covers the error return,
when some content may already be gone.

### Trade-off

Within one handle, a repeat check no longer re-hashes content. A separate process deleting blobs
while this handle is live was already outside the completeness check's guarantee: the check is a
read of the services at that moment, not a lease. The per-handle boundary and that limitation are
recorded in the delta spec and in the field doc comment.

## Risks / Trade-offs

- A handle that verified a node and then had its content deleted by another process would answer
  `true` from the memo. The same race already existed between two consecutive uncached checks, and
  every in-handle deletion clears the set.
- The set holds at most `MAX_VERIFIED_COMPLETE_NODES` keys (roughly 40 bytes each); at the bound
  the route pays the full check again rather than growing without limit.
- If the HTTP closure import is later moved onto a mutable handle, it can record the members it
  actually ingested and NAR-checked. This change does not do that because the import path is
  immutable end to end and its public signature is shared.

## Evidence

- `crates/crunch-store/src/handle.rs` unit tests:
  `cached_path_info_repeat_on_one_handle_reads_no_blobs`,
  `castore_node_verified_complete_rechecks_incomplete_nodes`,
  `castore_node_verified_complete_rechecks_an_absent_blob`, and
  `garbage_collection_clears_verified_complete_nodes`. They use a counting `BlobService` wrapper, so
  reads are observed directly rather than inferred from timing.
- The existing route suites must pass unchanged, pinning that hydration, reuse classification, and
  receipts are unaffected.
