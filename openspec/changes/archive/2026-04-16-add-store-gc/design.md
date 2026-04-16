# Design: Add store garbage collection

## Context

crunch stores four related kinds of local state:

- persisted `PathInfo` metadata
- castore blobs and directories
- optional exported filesystem outputs under the physical store directory
- persisted artifact and closure attestation sidecars under the state dir

Today that state grows monotonically. The repo already has the ingredients for a
safe collector: `PathInfo` references encode store-path reachability, and the
store layer already owns metadata and castore access. What is missing is a
stable root set, a mark-and-sweep walk, and an operator surface that makes
collection explainable.

## Goals / Non-Goals

**Goals:**

- retain requested outputs durably across process restarts
- reclaim unreachable `PathInfo`, exported paths, and castore content safely
- fail closed when retained-root reachability facts are missing or unreadable
- make GC inspectable before deletion through dry-run output
- keep GC logic inside `crunch-store`, not scattered through CLI code

**Non-Goals:**

- background daemon GC
- remote binary-cache cleanup
- policy based on age, LRU, or disk-pressure thresholds
- cross-machine root synchronization

## Decisions

### 1. Keep a durable root registry in state

**Choice:** store a durable GC root registry under the crunch state directory,
keyed by logical store path.

**Rationale:** GC safety starts with a stable root set. A single retained-root
record per logical path keeps the CLI simple (`pin <path>`, `unpin <path>`) and
avoids ambiguous multi-record deletion semantics.

**Implementation:** each retained-root record stores the logical store path plus
source metadata (`build`, `self-build`, `bootstrap`, or explicit pin) and
creation time as informational fields. Here `bootstrap` means the
`crunch bootstrap --fetch` entry point. Re-rooting an existing path updates
that record instead of creating a second retained-root entry.

### 2. Auto-root top-level requested outputs only

**Choice:** successful top-level outputs from `crunch build`, `crunch
self-build`, and `crunch bootstrap --fetch` become retained roots
automatically.
Dependencies do not.

**Rationale:** operators usually want the outputs they asked for to survive, but
keeping every dependency forever defeats GC. Top-level auto-rooting matches user
intent without turning the full transitive build graph into permanent roots.

**Implementation:** root registration happens after successful persistence of the
final output metadata. Failed builds never create roots.

### 3. Use mark from roots, then sweep metadata, exports, and castore

**Choice:** `crunch store gc` performs a two-phase reachability pass. First it
marks live store paths by traversing `PathInfo.references` from the root set.
Then it derives the live castore node set from the surviving `PathInfo` records
and sweeps unreachable metadata, exports, and castore content.

**Rationale:** `PathInfo` is the canonical graph for store reachability, while
castore content is an implementation detail shared by multiple paths. Splitting
the passes keeps reachability decisions deterministic and avoids accidental blob
deletes before metadata reachability is known.

**Implementation:** the collector records candidate deletions before mutation,
then deletes in a stable order: exported path removal, `PathInfo` removal,
attestation-sidecar cleanup, and castore sweep after the live-node set is
frozen. Castore content is deletable only when no surviving `PathInfo` still
references it. The collector snapshots rows before mutating them so it never
calls `put()` or other writes while iterating a live `list()` cursor on the
same backend.

### 4. Missing root reachability metadata aborts collection

**Choice:** GC aborts before deletion if any retained root lacks readable
reachability metadata.

**Rationale:** GC is destructive. If crunch cannot prove a retained root's live
closure, it should preserve everything rather than risk deleting live content.

**Implementation:** unreadable root-registry entries, missing `PathInfo` for a
retained root, or corrupted retained-root metadata produce an explicit GC error
before the sweep phase starts.

### 5. Require exclusive local store mutation during GC

**Choice:** manual GC requires exclusive local store-mutation access for the
lifetime of the run.

**Rationale:** a new output persisted between mark and sweep can look
unreachable even though a concurrent build is still finalizing it.

**Implementation:** `crunch store gc` acquires an exclusive local mutation lock
before mark starts and refuses to run if another local build or substitution
mutation is active.

### 6. Dry-run and reporting are first-class

**Choice:** dry-run is a normal execution mode, not an afterthought.

**Rationale:** GC is destructive. Operators need to inspect path counts, byte
counts, and root explanations before trusting it on a real store.

**Implementation:** `--dry-run` returns the same accounting and candidate list
as a real run, but performs no mutation. Human output summarizes roots kept and
bytes reclaimable; machine-readable output can be added later without changing
the reachability core.

### 7. Root management stays explicit

**Choice:** provide explicit root inspection and pin/unpin commands instead of
forcing operators to edit state files or rely on undocumented auto-rooting.

**Rationale:** once GC exists, operators need a supported way to retain or drop
important outputs intentionally.

**Implementation:** `crunch store roots` lists retained roots, `crunch store pin`
adds one, and `crunch store unpin` removes one. These commands mutate only the
root registry.

## Risks / Trade-offs

**[Over-retention]**
Auto-rooting top-level outputs may keep more store state than some operators
want.

**Mitigation:** keep explicit unpin support and leave aggressive policy work for
later changes.

**[Under-retention]**
If a result should survive but was never rooted, GC can delete it.

**Mitigation:** auto-root top-level outputs by default and make retained roots
visible through `crunch store roots`.

**[Castore accounting cost]**
Sweeping castore content requires traversing live nodes after the `PathInfo`
mark phase.

**Mitigation:** keep the marking core pure and bounded, then optimize storage
walks only after correctness is proven.
