## Context

crunch's `Builder` is generic over `BS: BlobService`. The production code in
`main.rs` and `bootstrap.rs` currently instantiates `MemoryBlobService`. All
other services (`RedbDirectoryService`, `RedbPathInfoService`) are already
disk-backed. Blob storage is the outlier.

snix-castore ships `ObjectStoreBlobService` which wraps the `object_store`
crate. It supports any backend (`file://`, S3, GCS, etc.) and handles
chunking, content-addressing, and deduplication internally. It's already
compiled as part of the snix-castore dependency — no new crates needed.

## Goals / Non-Goals

**Goals:**
- Persistent, disk-backed blob storage for production builds
- Bounded memory usage during large builds
- Cache hits across process restarts (blobs + PathInfo)
- No changes to crunch-build library code (it's already generic)

**Non-Goals:**
- Blob garbage collection (future work — prune unreferenced blobs)
- Remote blob backends (S3, etc. — future work)
- Changing the directory service (already persistent via redb)
- Changing test infrastructure (MemoryBlobService is fine for tests)

## Decisions

### 1. Use ObjectStoreBlobService with local filesystem backend

**Choice:** `ObjectStoreBlobService` configured with
`objectstore+file://{state_dir}/blobs/`

**Rationale:** Already available in the dependency tree. Handles chunking
(FastCDC, 256 KiB avg), content-addressed dedup, and sharded directory
layout (prevents too many files in one dir). The local filesystem backend
has no external dependencies.

**Alternative rejected:** Writing a custom `RedbBlobService`. Redb is
optimized for small key-value pairs, not multi-MB blobs. Large values
cause write amplification and compaction stalls. The object_store local
filesystem approach stores chunks as individual files — simple, debuggable,
and the OS page cache handles hot data.

**Alternative rejected:** Keeping MemoryBlobService and adding a flush-to-disk
step. This doesn't solve the bounded-memory requirement — blobs still
accumulate in RAM during the build. The fix needs to be at the storage
layer, not the export layer.

**Implementation:**

`ObjectStoreBlobService` fields are private and the `ServiceBuilder::build()`
method requires a `CompositionContext` with a `&'static Registry` — overkill
for our use case. Since we vendor snix-castore, add a direct constructor:

```rust
// In vendor/snix-castore/src/blobservice/object_store.rs
impl ObjectStoreBlobService {
    pub fn new_local(path: impl AsRef<std::path::Path>) -> std::io::Result<Self> {
        let local = object_store::local::LocalFileSystem::new_with_prefix(path.as_ref())
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        Ok(Self {
            instance_name: String::new(),
            object_store: Arc::new(local),
            base_path: object_store::path::Path::default(),
            avg_chunk_size: 256 * 1024,
        })
    }
}
```

Then in `main.rs` and `bootstrap.rs`, replace:

```rust
let blob_service = MemoryBlobService::default();
```

with:

```rust
let blob_dir = state_dir.join("blobs");
std::fs::create_dir_all(&blob_dir)?;
let blob_service = Arc::new(ObjectStoreBlobService::new_local(&blob_dir)?);
```

`Builder` is generic over `BS: BlobService + Clone`. `Arc<ObjectStoreBlobService>`
satisfies both (BlobService has `auto_impl` for Arc, Arc is Clone).

### 2. Blob storage location: `{state_dir}/blobs/`

**Choice:** `~/.local/state/crunch/blobs/` (sibling to `pathinfo.redb`)

**Rationale:** Follows XDG conventions (state data). Co-locating with
PathInfo makes the cache relationship obvious and simplifies any future
GC (scan PathInfo → mark referenced blobs → sweep unreferenced).

**Implementation:** The existing `state_dir()` function in main.rs returns
the base directory. Append `/blobs/`.

### 3. Keep MemoryBlobService for tests

**Choice:** No changes to test infrastructure.

**Rationale:** Tests need isolation, speed, and no side effects.
`MemoryBlobService` provides all three. The `Builder` generic constraint
means test code doesn't care which implementation is used.

### 4. Direct constructor on vendored ObjectStoreBlobService

**Choice:** Add `pub fn new_local(path)` to `ObjectStoreBlobService` in
vendored snix-castore. Bypasses the composition/registry system entirely.

**Rationale:** The `ServiceBuilder::build()` path requires a
`CompositionContext<'a>` with a `&'static Registry`. That's the snix
composition framework for declarative multi-service wiring — we don't use
it. A direct constructor is simpler, has no lifetime constraints, and
produces an owned value (not `Arc<dyn BlobService>`), which keeps the
`Builder` generic concrete.

**Alternative rejected:** Going through `TryFrom<Url>` +
`ServiceBuilder::build()`. Requires constructing a static Registry,
returns `Arc<dyn BlobService>` (loses concrete type), and adds ceremony
for no benefit.

## Risks / Trade-offs

**[Disk space growth]** → Blobs accumulate with no GC. Mitigation: document
that `~/.local/state/crunch/blobs/` can be deleted to reclaim space (forces
rebuild, but PathInfo GC is a separate future change).

**[First-build latency]** → Disk I/O is slower than RAM for small blobs.
Mitigation: OS page cache covers hot data. The latency difference is
negligible compared to actual build time.

**[ObjectStoreBlobService chunking overhead]** → FastCDC + blob index
files add slight overhead per blob. Mitigation: the 256 KiB avg chunk
size is well-tuned for build artifacts. Single-chunk blobs (< 512 KiB)
have minimal overhead.

**[ObjectStoreBlobServiceConfig API stability]** → The config struct and
`TryFrom<Url>` are snix internal APIs. Mitigation: we vendor snix-castore,
so we control the API.
