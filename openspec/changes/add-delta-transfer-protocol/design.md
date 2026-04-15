# Design: Add delta transfer protocol

## Context

Crunch already has most of the raw ingredients that made DeltaNAR attractive:

- persistent `PathInfo` for logical store membership and references
- castore directory nodes for subtree identity
- castore blobs with BLAKE3 identities and content-defined chunking
- a near/far blob read pattern in vendored `snix-castore`
- native artifact attestations and signed `PathInfo`

What crunch lacks is a protocol that lets a remote sender ask, "which parts of
this closure do you already have?" and then ship only the missing pieces.

That protocol should fit crunch's own store model. Crunch does not need a
second storage format that re-encodes castore state back into NAR-shaped
messages just to regain directory, file, and chunk reuse.

## Goals / Non-Goals

**Goals:**

- define a crunch-native delta transfer boundary for remote substitution and
  future deployment-style workflows
- reuse existing castore blob and directory identities instead of inventing a
  parallel chunk namespace
- make logical store prefix compatibility explicit during negotiation
- preserve current signature and attestation trust rules
- keep full-artifact substitution as a safe fallback

**Non-Goals:**

- replace every remote cache path with delta transfer on day one
- require wire compatibility with DeltaNAR or Nix's DNAR format
- add a new general-purpose deployment CLI in this change
- change crunch's choice of BLAKE3 for content identity

## Decisions

### 1. Delta transfer is castore-native, not NAR-native

**Choice:** crunch delta transfer will describe directory nodes, whole blobs,
and blob chunks using crunch's existing castore identities. It will not define
NAR archives as the primary transfer unit.

**Rationale:** crunch already stores data as directory and blob nodes with
content-defined chunk metadata. Re-serializing that data into a second NAR-only
format would duplicate logic, create two dedup stories, and make later
verification harder.

**Implementation:** future work should derive transfer plans from `PathInfo`
plus castore node metadata. A receiver that already has a matching directory or
blob digest should reuse that object directly.

### 2. Negotiation pins protocol version and chunk profile

**Choice:** delta-capable peers must negotiate a protocol version and chunk
profile before reuse planning starts. The initial profile is FastCDC with 128
KiB minimum, 256 KiB average, and 512 KiB maximum chunk size. Chunk identity is
BLAKE3 of finalized chunk bytes.

**Rationale:** content-defined chunk reuse only works when both sides cut the
same byte ranges and name those ranges the same way. Leaving the chunk profile
implicit would make reuse brittle and hard to debug.

**Implementation:** version and chunk profile must be explicit negotiation
fields so later protocol revisions can add new profiles without silently
changing behavior.

### 3. Receiver state is advertised as scoped logical store facts

**Choice:** the receiver will advertise a bounded compatibility manifest built
from logical store facts: configured store prefix, present output identities,
present directory digests, present blob digests, and reusable chunk digests.
That manifest is scoped to the requested output or closure transfer, not a
whole-store inventory.

**Rationale:** crunch supports configurable logical store prefixes, and its
cache truth is `PathInfo` plus castore content, not exported host paths. A
whole-store manifest would be too large and would fight the streaming goal.

**Implementation:** manifest construction must use metadata and content probes,
not content downloads. `PathInfo` entries without backing castore content are
absent for reuse planning. Prefix mismatch is a hard compatibility failure.

### 4. Reuse is attempted coarsest-first, including cross-output reuse

**Choice:** delta planning will attempt reuse in this order: whole artifact,
unchanged directory subtree, whole blob, then content-defined chunk. Reuse may
cross output boundaries within the same requested closure.

**Rationale:** whole-object reuse is cheaper to describe and verify than a long
list of fine-grained chunk edits. DeltaNAR's main insight is sound, but crunch
can express it directly in its castore hierarchy and does not need to limit
reuse to a single output at a time.

**Implementation:** if the receiver already has a root output or referenced
subtree, the sender should skip finer-grained planning for that object. Chunk
planning is only for the remaining changed blobs. Cross-output blob reuse is a
MUST within one transfer session, so the sender tracks which blob digests have
already been sent. That tracking state is bounded by the number of unique blob
digests in the requested transfer, not by total bytes.

### 5. Interaction is sender-described and receiver-confirmed

**Choice:** the first delta interaction is a staged exchange:

1. the receiver requests one or more logical outputs or a closure root from the
   cache authority,
2. the sender replies with the protocol version, chunk profile, and the scoped
   candidate identities for that transfer,
3. the receiver replies with which of those advertised identities it already
   has with backing castore content,
4. the sender streams only the missing content plus the final signed `PathInfo`.

**Rationale:** the receiver cannot scope its manifest intelligently until the
sender says which identities are candidates for reuse. This sequence keeps the
manifest bounded to the requested transfer while preserving sender-driven
planning.

**Implementation:** the first HTTP transport should reflect this staged flow,
with candidate identities scoped to the requested outputs or closure roots.

### 6. Delta planning operates on finalized content only

**Choice:** delta transfer operates on finalized output content, after any CA
self-reference rewriting or output-path fixups have already happened.

**Rationale:** chunk identities are only stable once the output bytes are final.
Planning against provisional content would make self-referential outputs look
changed when their finalized bytes are actually identical.

**Implementation:** senders build reuse plans from persisted finalized store
content and finalized `PathInfo`, not from pre-finalization builder scratch
state.

### 7. First transport extends the current binary-cache HTTP authority

**Choice:** the first transport shape extends the existing remote-cache HTTP
flow with delta negotiation and streaming endpoints under the same authority as
ordinary substitution.

**Rationale:** crunch already has a remote substitution path and trust model
centered on cache authorities. Reusing that boundary keeps fallback simple and
avoids inventing a second connection model before it is needed.

**Implementation:** later work may refactor transport internals, but the first
version should fit the current binary-cache topology.

### 8. Delta transfer stays under existing substitution trust rules

**Choice:** delta transfer is an optimization under substitution, not a new
trust model.

**Rationale:** a faster transport must not change what counts as an accepted
cache hit. Crunch already has signed `PathInfo` and native artifact attestation
rules for accepted substitutions.

**Implementation:** after assembling the received chunks and blobs, the
receiver must verify that the reconstructed castore root node digest matches the
root node referenced by the final signed `PathInfo`. Only then may the logical
store path be accepted. Accepted delta hits must still yield the same
artifact-attestation outcome as ordinary substitution.

### 9. Partial transfer keeps verified content, not session state

**Choice:** successfully verified chunks and blobs from an interrupted transfer
stay in castore, but incomplete transfer-session state is discarded.

**Rationale:** castore objects are content-addressed and independently valid, so
keeping them improves the next attempt without needing resumable session
bookkeeping. The incomplete logical output must still remain unaccepted.

**Implementation:** retries can benefit from the retained content through the
next compatibility-manifest exchange.

### 10. Full-artifact substitution remains the fallback

**Choice:** delta transfer is negotiated opportunistically. If the peer does
not support it, manifest exchange fails, or reuse is not worthwhile, crunch
falls back to existing full-artifact substitution behavior.

**Rationale:** crunch already has a working substitution path. Delta support
should extend that path, not replace it with an all-or-nothing design.

**Implementation:** the binary-cache layer should treat delta transfer as an
optional capability advertised by a trusted remote source.

### 11. Transfer must stay streaming and bounded

**Choice:** sender and receiver must stream transfer state with bounded memory
that scales with active blobs, directories, and chunk windows rather than total
closure size.

**Rationale:** the entire point is to save bandwidth on large closures without
introducing a memory cliff. Crunch's current blob storage already expects
streaming chunk ingress.

**Implementation:** future work should avoid materializing the whole closure or
whole delta plan in memory when a streamed walk is enough.

## Risks / Trade-offs

**[Protocol complexity]**
A reuse-aware protocol is more complex than plain full-artifact fetch.

**Mitigation:** keep fallback simple and reuse existing castore abstractions.

**[Prefix fragmentation]**
Peers with different logical store prefixes cannot safely share delta state.

**Mitigation:** make prefix compatibility explicit up front and fail fast.

**[Manifest skew]**
Receiver metadata can claim reusable content that no longer has backing castore
objects.

**Mitigation:** require castore-backed presence checks before advertising reuse.

**[Limited first deployment]**
A first implementation may cover substitution before broader deployment tooling.

**Mitigation:** scope that intentionally. The protocol boundary should still be
useful for later push or peer-to-peer workflows.

## Verification

- **Unit:** manifest construction from `PathInfo` plus castore metadata without
  content download, including stale metadata treated as absent
- **Unit:** protocol negotiation covers version, chunk profile, and prefix
  mismatch rejection
- **Unit:** coarsest-first planner covers whole-output hit, unchanged subtree
  hit, whole-blob hit, chunk-only hit, and cross-output blob reuse
- **Unit:** finalized-content planning for CA outputs uses post-rewrite bytes
- **Integration:** build output version 1, pre-seed a fresh receiver state with
  reusable content, serve a delta-capable cache for version 2, substitute, and
  verify only missing content is transferred
- **Integration:** same setup with untrusted signatures rejects the delta hit
  and preserves ordinary trust behavior
- **Integration:** a legacy cache with no delta endpoints falls back cleanly to
  ordinary full-artifact substitution
