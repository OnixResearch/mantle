## Why

Mantle operates on exactly one local store per invocation. A common
deployment shape — a trusted read-only base store shared across users plus a
per-user writable overlay that holds only rebuilt paths — cannot be expressed
today. The closest existing approximation is the ordered `--substituters`
chain, but substitution is cache-with-backfill: every miss copies the full NAR
into the local store, the local store grows monotonically toward the base,
there is no shadowing, and trust is not layered.

This change introduces overlay store composition (ADR 0012): a writable
overlay `StoreHandle` layered over one or more read-only base stores, both
sharing the same logical store prefix, presenting a single merged
read-through view to the build pipeline and CLI.

## What Changes

- Add a read-only-far / no-backfill mode to the vendored
  `Cache` (DirectoryService, PathInfoService) and `CombinedBlobService`
  combinators so reads pass through the base without mutating the overlay.
- Add overlay composition to `crates/crunch-store`: a writable overlay
  `StoreHandle` over read-only base(s), with write routing to the overlay
  only, trust-provenance-per-layer, and cross-layer GC safety.
- Add a global `--base-store <state-dir>` CLI option (repeatable, ordered)
  that stacks read-only bases below the writable local store.
- Route sandbox input resolution (`resolve_and_ingest_sources`,
  `cached_node_for_path`) through the composed handle so bwrap sees a merged
  castore view transparently.
- Extend attestation synthesis (`crunch-attestation` / `crunch-store`) to
  record the producing layer (overlay vs base) on artifact and closure
  attestations, and make `crunch attest verify` honor layered trust.
- Extend overlay GC to record base references in the overlay's reachability
  set so an overlay GC cannot dangle a path whose content is only reachable
  through the base.

## Impact

- **Files**: `crates/crunch-store/src/handle.rs`, `crates/crunch-store/src/gc.rs`,
  `crates/crunch-store/src/attestation.rs`, `crates/crunch-store/src/closure.rs`,
  `crates/crunch-build/src/orchestrate.rs`, `vendor/snix-castore/src/directoryservice/combinators.rs`,
  `vendor/snix-castore/src/blobservice/combinator.rs`, `vendor/snix-store/src/pathinfoservice/cache.rs`,
  `src/main.rs`, `crates/crunch-attestation-core/src/` (provenance layer field),
  `adr/0012-overlay-store-composition.md`.
- **Testing**: positive tests for read-through (base hit does not mutate
  overlay), write routing (puts land in overlay only), shadowing (overlay
  path shadows base), cross-layer GC (base-referenced path survives overlay
  GC), and layered attestation. Negative tests for prefix mismatch (hard
  error), base write attempt (rejected), and shadowed-path trust inheritance
  (overlay swap does not inherit base signature).
- **Compatibility**: existing single-store invocations are unchanged; the new
  combinators default to today's backfill-on-read behavior. ADR 0003's
  prefix-in-hash invariant is preserved (both layers share one prefix).
