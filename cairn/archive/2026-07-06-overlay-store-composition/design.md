## Design

### Architecture

Overlay composition layers two `StoreHandle` instances — a writable overlay
(local store) over a read-only base — behind the existing three service
traits (`BlobService`, `DirectoryService`, `PathInfoService`). The build
pipeline and CLI see one composed handle; the composition is transparent
above and explicit below.

```
CLI --base-store <state-dir>   (repeatable, ordered)
  -> StoreConfig { base_state_dirs: Vec<PathBuf>, ... }
  -> StoreHandle::open_overlay(config)
  -> overlay services wrapped around base services via combinators
  -> Builder { store: composed handle }
  -> orchestrate.rs reads through composed handle
  -> sandbox sees merged castore view
```

### The composition seam

The vendored snix crates already ship `Cache<Near, Far>` / `CombinedBlobService<Near, Far>`
combinators for all three service traits. Their current semantics are
"near-first, far-fallback, **backfill into near**" — i.e. cache behavior.
An overlay must instead be "overlay-first, base-fallback, **no backfill on
read**" plus write-routing-to-overlay and trust-provenance-per-layer.

The change splits responsibilities:

- **snix (vendored) owns the read primitive**: extend the combinators with a
  `ReadOnlyFar` / `no_backfill` mode so `get` consults far on a near miss but
  does not `put` the result back into near. This is a small contained edit to
  already-vendored code. Default behavior stays backfill-on-read so no current
  caller regresses.
- **crunch-store owns the overlay product contract**: a composed
  `StoreHandle` that wires overlay services over base services using the
  no-backfill combinators, routes all writes to the overlay, tracks the
  producing layer for trust/attestation, and coordinates GC.

### Functional core / imperative shell boundary

- **Pure core** (testable without I/O): the overlay read-ordering decision
  (given overlay-has / base-has booleans, which layer serves a `get`), the
  write-routing decision (writes always target overlay), the prefix-match
  validation (overlay and base must share `store_dir`), the trust-layer
  resolution (given a path present in both layers, which signatures govern),
  and the GC-reachability rule (given a path whose only content is in the
  base, is it live in the overlay). These are small pure functions over
  in-memory inputs.
- **Imperative shell**: `StoreHandle::open_overlay` wires real redb/memory
  services, opens the base read-only, constructs the combinators, and owns
  the `StoreMutationGuard` for the overlay (the base takes no mutation lock
  because it is read-only).

### Read-through (no backfill) — the overlay-vs-cache distinction

The defining difference from the existing substituter chain and from the
default `Cache` combinator. A read that hits the base returns the base value
directly; it MUST NOT copy the blob/directory/PathInfo into the overlay. This
keeps the overlay thin. Only explicit outputs — builds, `store pull`, `store
import` — write to the overlay.

### Trust provenance per layer

Each `CacheHit` / consumed node carries a `StoreLayer { Overlay, Base }` tag.
A path present in the overlay shadows the base; the overlay's `PathInfo`
signatures (or absence) govern for that path, NOT the base's. A path served
from the base inherits base trust. Attestation synthesis records the layer on
`ArtifactProvenance` so `crunch attest verify` can distinguish base-sourced
from overlay-sourced evidence. A swapped blob in the overlay does not inherit
the base's signature trust.

### Cross-layer GC

Overlay GC (`crates/crunch-store/src/gc.rs`) walks the overlay's own
PathInfo + castore. A path whose content is only reachable via a base
reference (the overlay PathInfo references a base-sourced path) MUST survive
overlay GC — removing it would dangle the overlay closure. The overlay
records base-referenced paths in its reachability set during the mark phase.
The base is never GC'd by Mantle; it is owned externally and opened read-only.

### Sandbox mount

`resolve_and_ingest_sources` and `cached_node_for_path` in
`crates/crunch-build/src/orchestrate.rs` already read from the `StoreHandle`
they are given. Because composition is at the service-trait layer, the merged
castore view is presented transparently — no new bwrap wiring is needed. The
only required change is ensuring the orchestrator receives the composed
handle, which flows from `StoreConfig.base_state_dirs` through `open_overlay`.

### Same-prefix invariant

Both overlay and base MUST share `store_dir`. A prefix mismatch is a hard
configuration error at `open_overlay` time. This preserves ADR 0003's
prefix-in-hash invariant: a derivation planned under the shared prefix
produces hashes valid against both layers, so a base-sourced input is
consumable by an overlay build and vice versa.

### Rejected design choices

- **New overlay combinator in crunch-store, ignoring vendored combinators**:
  rejected — duplicates the near/far read path that snix already provides and
  would drift from upstream. Reusing the combinators with a no-backfill mode
  keeps one read primitive.
- **Backfill-on-read as overlay behavior**: rejected — that is cache
  semantics; it grows the overlay toward the base and defeats the
  thin-overlay deployment shape.
- **Multi-prefix-at-once**: rejected (ADR 0003) — hashes are prefix-dependent,
  so layers with different prefixes cannot share inputs.

### Open questions

- Whether the no-backfill combinator mode should be a separate type
  (`Overlay<Near, Far>`) or a config flag on `Cache`. Lean toward a config
  flag to avoid combinatorial type proliferation across the three traits;
  revisit at implementation time.
- Whether multiple `--base-store` bases stack with `OR` semantics (any base
  that has the path serves it) or with a defined order (first declared base
  consulted first). Lean toward ordered, matching the substituter chain.
