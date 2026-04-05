## Context

The Builder currently maintains `built_outputs: HashMap<String, PathInfo>`
and `output_nodes: HashMap<StorePath, Node>` as session-local state.
Cache checks use `all_outputs_exist()` which calls `path.exists()` on
the filesystem. When a cached output is found, `load_cached_outputs()`
re-ingests it from disk (computing blob digests and NAR hash from
scratch) to reconstruct PathInfo.

snix-store ships `RedbPathInfoService` which stores PathInfo as
protobuf-encoded bytes in a redb key-value database, keyed by the
20-byte store path digest. It implements the `PathInfoService` trait
(get, put, list). The code is vendored and compiles.

The `PathInfoService` trait from snix-store uses 20-byte digests
(the compressed hash from the store path) as keys. This matches
`StorePath`'s internal digest representation.

## Goals / Non-Goals

**Goals:**
- PathInfo survives process restart
- Cache hits return stored PathInfo without re-ingesting from disk
- `crunch store` subcommand for inspection and verification
- Graceful fallback if database is broken

**Non-Goals:**
- Garbage collection (separate change)
- Signing PathInfo (future)
- Remote PathInfoService (gRPC, HTTP — snix-store has these but
  we don't need them yet)
- Migrating existing filesystem-only "cached" outputs into the
  database (they'll be rebuilt and recorded on next build)

## Decisions

### 1. Use RedbPathInfoService directly

**Choice:** Instantiate `RedbPathInfoService` in the CLI layer
and pass it to `Builder` as a generic parameter bounded by
`PathInfoService`.

**Rationale:** The code exists, compiles, and is tested in
snix-store's own test suite. Redb is already a transitive
dependency (snix-castore's `RedbDirectoryService` uses it).
No new dependencies.

**Alternative:** Write a custom SQLite-based store. Rejected —
redb is simpler (single-file, no SQL, already in the dep tree)
and the PathInfoService trait + protobuf serialization are already
wired up.

**Implementation:**
```rust
let pathinfo_service = RedbPathInfoService::new(
    "crunch".to_string(),
    RedbPathInfoServiceConfig {
        path: Some(state_dir.join("pathinfo.redb").to_str().unwrap().into()),
        read_only: false,
        cache_size: None,
    },
).await?;
```

### 2. Builder generic over PathInfoService

**Choice:** Add `PIS: PathInfoService` as a type parameter to
`Builder<BS, DS, BServ, PIS>`. The Builder calls `PIS::get()`
for cache checks and `PIS::put()` after builds.

**Rationale:** The trait is already defined in snix-store.
Generic parameter keeps tests flexible — unit tests can use
`LruPathInfoService` (in-memory, no disk).

**Alternative:** Store `Box<dyn PathInfoService>` instead of a
generic. Works but adds dynamic dispatch overhead on every cache
check. The generic is zero-cost and consistent with how `BS`,
`DS`, and `BServ` are already parameterized.

### 3. Two-condition cache check

**Choice:** Cache hit requires `PathInfoService::get()` returning
`Some` AND the filesystem path existing. Either condition alone is
a miss.

**Rationale:**
- PathInfo without file: store was garbage collected or manually
  deleted. Rebuilding is correct.
- File without PathInfo: path might be from Nix, a manual copy,
  or a previous crunch version without persistence. We can't
  trust it — no references, no deriver, no NAR hash. Rebuilding
  records it properly.

This is stricter than v0 (which trusts any existing path).
The trade-off is one-time rebuilds of existing cached outputs.
Acceptable.

### 4. PathInfo replaces load_cached_outputs re-ingestion

**Choice:** When a cache hit occurs (PathInfo found + file exists),
return the stored PathInfo directly. Don't call `ingest_path` or
`calculate_nar`. Extract the `Node` from the stored PathInfo for
`output_nodes`.

**Rationale:** Re-ingesting a cached output is slow — it reads
every byte, hashes it, and stores it in the blob service. For
large outputs (hundreds of MB), this dominates build time. With
stored PathInfo, it's a single redb read.

**Risk:** The stored PathInfo could be stale if someone modifies
the output on disk. The `crunch store verify` command exists for
this. We don't verify on every cache hit (too expensive). The
assumption is the store is immutable once written.

### 5. State directory layout

**Choice:**
```
$XDG_STATE_HOME/crunch/
├── logs/          # build logs (existing)
└── pathinfo.redb  # PathInfo database (new)
```

**Rationale:** Follows XDG conventions. Logs already go here.
Single redb file is self-contained (no WAL, no multi-file
format). Easy to back up, easy to delete for a clean slate.

### 6. crunch store subcommand

**Choice:** Add `crunch store list|info|verify` as a read-only
subcommand. Opens the database in read-only mode.

**Rationale:** Users need to inspect what crunch knows about.
`list` for an overview, `info` for details, `verify` for
integrity checks. Read-only mode means it can run concurrently
with builds.

**Implementation:** Opens `RedbPathInfoService` with
`read_only: true`. Iterates via `PathInfoService::list()`.
For `verify`, re-computes NAR hash by ingesting from disk and
compares.

## Risks / Trade-offs

**[One-time rebuild cost]** Existing cached outputs (built by
v0 without persistence) will be rebuilt on first run with the
new code. After that, they're recorded and cached properly.

**[Redb file growth]** The database grows with every build.
Without GC, it accumulates PathInfo for outputs that may have
been deleted from the store. `crunch store verify` can detect
these, but automated cleanup is deferred to the GC change.

**[Protobuf dependency]** `RedbPathInfoService` stores PathInfo
as protobuf bytes. This is already the case in the vendored code
and protobuf is already a build dependency (prost). No new dep.
