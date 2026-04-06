## Context

Builder owns 4 service generics and directly calls blob/directory/pathinfo
methods throughout its 2700 lines. Store initialization lives in main.rs
free functions. Cache logic, realization, CA mappings, and store queries
are all inside Builder or main.rs.

## Goals / Non-Goals

**Goals:** Extract store operations into a standalone crate with trait
boundaries. Reduce Builder's surface area. Make store operations testable
and embeddable independently.

**Non-Goals:** Change the storage backends (redb, object-store). Change
the PathInfo format. Add GC (separate change).

## Decisions

### 1. StoreHandle as a concrete struct, not a trait

**Choice:** `StoreHandle` is a struct holding `Arc<dyn BlobService>`,
`Arc<dyn DirectoryService>`, `Arc<dyn PathInfoService>`, and optionally
`Arc<dyn PathInfoService>` for remote cache.

**Rationale:** The snix traits are already dynamic dispatch points. Adding
another trait layer on top adds indirection without benefit. A concrete
struct that holds trait objects is simpler to construct and pass around.

**Alternative:** Generic `Store<BS, DS, PIS>` — rejected because it
propagates generics through every caller. The whole point is to stop
the generic proliferation.

### 2. CacheService as a method on StoreHandle, not a separate trait

**Choice:** `store.check_cache(drv, derivation)` is a method, not a
separate trait implementation.

**Rationale:** Cache checking always needs blob + directory + pathinfo +
remote. A separate trait would just forward to StoreHandle's internals.
The method keeps it simple. If we later need pluggable cache strategies,
we can extract then.

### 3. Move export.rs and ca_mapping.rs into crunch-store

**Choice:** Both files move wholesale.

**Rationale:** They're store operations with no build-engine knowledge.
`export_castore_to_disk` only needs BlobService + DirectoryService.
`CaMappings` is a persistent JSON file keyed by drv path.

### 4. Builder takes StoreHandle, not 4 generics

**Choice:** `Builder<BServ>` with a `store: StoreHandle` field.

**Rationale:** The only remaining generic is `BuildService` — that's the
actual dispatch point for sandbox vs fetch vs remote builds. Everything
else goes through the store handle.

**Risk:** `StoreHandle` uses dynamic dispatch. Performance impact is
negligible — store operations are I/O-bound, not CPU-bound.

## Risks / Trade-offs

**[Migration size]** Every file that touches Builder's generics changes.
Mitigated by doing this change before any others (it's the foundation).

**[Test changes]** Tests that construct Builder with explicit types need
to construct StoreHandle instead. `MemoryBlobService` etc. still work —
they implement the same traits.
