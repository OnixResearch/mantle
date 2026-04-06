## Why

crunch uses `MemoryBlobService` for all blob storage — a `HashMap<B3Digest, Vec<u8>>`
in RAM. Every file from every build output lives entirely in memory. This causes
three problems:

1. **Memory pressure on large builds.** Compiling crunch itself produces hundreds
   of megabytes of object files, libraries, and binaries. All of it accumulates
   in the HashMap. Under memory pressure the process gets OOM-killed mid-export,
   leaving directories created on disk but files empty or missing (the "empty
   dirs" symptom noted in the napkin).

2. **No persistence across sessions.** PathInfo is persisted in redb, but blobs
   vanish when the process exits. On the next run `castore_has_content` fails →
   forced rebuild of everything, even if nothing changed. The persistent
   PathInfo becomes dead weight.

3. **Unbounded growth within a session.** Each build appends to the HashMap.
   Multi-package builds accumulate all intermediate outputs, even though only
   root outputs need export. The castore-only optimization (intermediate deps
   skip disk export) helps disk I/O but not RAM.

The directory service already uses `RedbDirectoryService` (on-disk, persistent).
Blob storage is the only component still in-memory.

## What Changes

Replace `MemoryBlobService` with `ObjectStoreBlobService` backed by a local
filesystem directory for production use (main.rs, bootstrap.rs). Tests keep
`MemoryBlobService` for speed and isolation.

- **Blob storage directory**: `~/.local/state/crunch/blobs/` (same parent as
  `pathinfo.redb`). Created on first use.
- **Chunked storage**: `ObjectStoreBlobService` uses FastCDC chunking (256 KiB
  avg). Large files are split into content-defined chunks, enabling cross-build
  dedup when the same libraries appear in multiple outputs.
- **Persistent cache**: blobs survive process restarts. Combined with the
  existing PathInfo redb + `castore_has_content` check, previously-built
  outputs are fully cached without rebuilding.
- **Bounded RAM**: blob data flows through a 64 KiB buffer during
  ingest/export, not accumulated in a HashMap.

## Capabilities

### New Capabilities
- `persistent-blob-store`: Blob data persists across crunch invocations,
  matching PathInfo persistence. Full build cache without Nix.
- `bounded-memory-builds`: Large builds no longer accumulate all blob data
  in RAM. Memory usage proportional to concurrent build count, not total
  output size.

### Modified Capabilities
- `castore-cache-check`: `castore_has_content` now hits disk-backed blobs,
  so cache hits work across sessions (previously always missed after restart).

## Impact

- **Files**: `src/main.rs`, `src/bootstrap.rs` — swap MemoryBlobService →
  ObjectStoreBlobService construction. `crates/crunch-build/` unchanged (generic
  over `BS: BlobService`).
- **APIs**: No API changes. `Builder` is already generic over blob service.
- **Dependencies**: `object_store` already in the dependency tree via
  snix-castore. `ObjectStoreBlobService` is already compiled. No new deps.
- **Testing**: Unit/integration tests keep MemoryBlobService. One new
  integration test verifies blob persistence across Builder instantiations.
- **Disk usage**: `~/.local/state/crunch/blobs/` will grow with build history.
  No automatic GC in this change (future work).
