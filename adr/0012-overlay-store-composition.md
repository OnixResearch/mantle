# ADR 0012: Overlay store composition

## Status

Accepted and implemented (2026-08-04)

## Context

Mantle currently operates on exactly one local store per invocation. `StoreConfig`
(`crates/crunch-store/src/handle.rs`) holds one `state_dir`, one `output_dir`,
one logical `store_dir` prefix, and an ordered list of remote substituter URLs.
`StoreHandle::open` builds a single blob service, a single directory service,
and a single pathinfo service, with a *separate* `remote_pathinfo`
(`NixHTTPPathInfoService`) used only for narinfo substitution.

This single-store model is operationally trivial but cannot express a common
deployment shape: a **trusted read-only base store** shared across users or
machines, plus a **per-user writable overlay** that holds only the paths the
user actually built or modified. Today the closest approximation is the
ordered `remote_cache_urls` substituter chain, but that is a *cache + backfill*
path, not an overlay:

- Substitution copies the entire NAR into the local store on every miss
  (the local store grows monotonically toward the base).
- There is no shadowing: a path present in both is served from local with no
  way to say "the base is authoritative, the overlay only overrides what it
  explicitly rebuilt."
- Trust is not layered: a substituted path inherits the remote's signature
  trust regardless of which layer produced it.

The vendored snix crates *already* ship `Cache<Near, Far>` combinators for
`DirectoryService` (`vendor/snix-castore/src/directoryservice/combinators.rs`),
`BlobService` (`CombinedBlobService`, `vendor/snix-castore/src/blobservice/combinator.rs`),
and `PathInfoService` (`vendor/snix-store/src/pathinfoservice/cache.rs`). All
three implement "ask near first, if missing ask far, and **insert the result
into near**" — i.e. cache-with-backfill. That backfill-on-read is the defining
behavior of a *cache*, and it is precisely what an *overlay* must NOT do: an
overlay reads through the base transparently while keeping the writable layer
thin, and only writes to the overlay on explicit build/substitution output.

ADR 0003 (configurable store prefix) established that the logical store prefix
is baked into derivation hashes via `to_aterm_bytes_with_store_dir` and the
`_with_store_dir()` family. A multi-prefix-at-once design (two logical
namespaces such as `/mantle/store` and `/nix/store` simultaneously) would
break hash sharing between layers and is explicitly rejected here.

## Decision

Introduce **overlay store composition** as a first-class Mantle store mode: a
writable overlay `StoreHandle` layered over a read-only base `StoreHandle`,
both sharing the **same logical store prefix**, presenting a single merged
view to the build pipeline and CLI.

### Composition model

- **Read ordering**: overlay-first, then base (read-through). A `get` on any
  of the three service traits consults the overlay; on a miss it consults the
  base; on a base hit it returns the base value **without copying it into the
  overlay**.
- **Write routing**: all puts (`put`, `put_multiple_start`, pathinfo `put`,
  blob writes, `persist_and_export_signed_output`) route to the overlay only.
  The base is opened read-only and MUST reject writes.
- **No backfill on read**: the distinction from the vendored `Cache`
  combinator. The overlay stays thin; only explicit build/substitution output
  lands in it.
- **Same prefix invariant**: both layers MUST share the same `store_dir`.
  Composition with mismatched prefixes is a hard configuration error.

### Where the composition lives

- **Read-through + write-routing primitive**: extend the vendored
  `Cache`/`CombinedBlobService` combinators with a *read-only-far / no-backfill*
  mode (a small, contained change to already-vendored code) so directory and
  blob layers read through the base without mutating the overlay.
- **Overlay semantics owned by crunch-store**: trust-provenance-per-layer,
  cross-layer GC safety, attestation routing, and the `--base-store` CLI
  declaration live in `crates/crunch-store`, not in vendored snix. snix owns
  the generic near/far read; Mantle owns the overlay product contract.

### Trust provenance

Each consumed path, blob, or directory carries a layer tag (`Overlay` or
`Base`). A path shadowed in the overlay does **not** inherit the base's
signature trust: if the overlay has a different `PathInfo` for the same store
path, the overlay's signatures (or absence) govern. Attestation synthesis
records which layer produced the artifact so `crunch attest verify` can
distinguish base-sourced from overlay-sourced evidence.

### GC coordination

- The overlay's GC MUST NOT remove a path whose content is only reachable via
  a base reference (the overlay would then dangle).
- The base is owned externally; Mantle never GCs the base. The overlay MAY
  record base references in its own GC reachability set so an overlay GC does
  not break a closure that transparently reads through the base.

### Sandbox mount

`resolve_and_ingest_sources` / `cached_node_for_path` in
`crates/crunch-build/src/orchestrate.rs` read from the composed
`StoreHandle`. Because composition is at the service-trait layer, the sandbox
mount path sees a single merged castore view transparently — no new bwrap
wiring is needed beyond routing input resolution through the composed handle.

### CLI surface

A new global `--base-store <state-dir>` option declares one read-only base.
The option is repeatable. Declaration order sets read precedence.

`mantle store list` and `mantle store info` show the selected layer.
Their JSON reports also include descriptors, generations, trust policy, signer names, and no-backfill status.

### Base admission and trust

Each base must be a direct directory. Symlinks and special files are rejected.
The base directory and all members must have no write permission.

Each base contains `store-identity.json`. This file binds the logical prefix, state schema, and trust policy.

Each base also contains `overlay-trusted-public-keys` or `signing-key`.
Mantle verifies each selected PathInfo signature with the accepted Ed25519 keys for that layer.
Signature presence alone is not sufficient.

Mantle computes a deterministic BLAKE3 generation identity over bounded state observations.
The identity includes ordered paths, member kinds, lengths, and content digests.
Mantle revalidates this identity before reads, execution, GC, and output admission.
Route reports bind selected base paths to the matching descriptor and generation.
Build reports record the overlay plan, base generations, selected layers, and shadows.
Attestation envelopes record the selected artifact or closure layers.

### Operations and rollback

GC planning includes exact layer ownership and cross-layer reachability.
GC can mutate only overlay PathInfo, directory, blob, export, attestation, root, and mapping state.
A base-to-overlay reference is invalid and blocks GC.

To roll back, remove all `--base-store` options and reopen the writable store.
This action restores single-store behavior. It does not copy base content into the overlay.

### Non-claims

The composition report does not prove permanent base immutability.
A generation identity does not prove whole-database atomicity or source correctness.
Layer trust does not prove content correctness, release eligibility, or mutation authority.
No-backfill evidence does not authorize writes to a base.

## Consequences

- **Shared base, thin overlay** becomes a supported deployment: many users
  mount one trusted base and keep per-user state small.
- **Vendored combinators gain a mode flag**. Existing `Cache`/`CombinedBlobService`
  callers default to backfill-on-read (today's behavior), so no current caller
  regresses.
- **Trust model gains a layer dimension.** `crunch attest verify` and
  `persist_and_export_signed_output` must record and honor provenance layer;
  a base-sourced path is trusted by the base's policy, an overlay path by the
  overlay's.
- **GC gets a cross-layer reachability rule** the single-store model did not
  need. Implemented in crunch-store, scoped to the overlay's own state.
- **No path invariance change**: both layers share a prefix, so derivation
  hashes, ATerm, and CA provisionals remain consistent with ADR 0003. A base
  also needs accepted identity, trust-key, permission, and generation facts.
- **Substituter chain is orthogonal.** `--substituters` still governs remote
  narinfo fetch for paths missing from *both* layers; the overlay just adds a
  local read-only tier below the local store and above the remote substituters.

## Alternatives Considered

**Multi-prefix-at-once (Variant B)**: run `/mantle/store` and `/nix/store`
simultaneously. Rejected — derivation hashes are prefix-dependent (ADR 0003),
so a path built under one prefix is not consumable as an input under another.
This is "two independent stores," not composition.

**Multi-backend, single prefix (Variant A)**: several writable backends
behind one prefix with merge-on-read. Rejected as the primary target because
the writable-shadow / read-only-base split (Variant C) is the deployment shape
users actually want; Variant A's multiple-writer merge has no clear coherence
model for disagreeing `PathInfo` and no shadow semantics.

**Substituters as the only tiering**: rejected because substitution is
backfill-on-miss, not read-through. It grows the local store toward the base
and provides no shadowing or layered trust.

**New overlay combinator in crunch-store, ignoring vendored combinators**:
rejected because the vendored `Cache`/`CombinedBlobService` already implement
the near/far read path; duplicating it would drift from upstream. Reusing them
with a no-backfill mode keeps the read primitive in one place.
