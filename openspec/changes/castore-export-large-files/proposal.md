# Castore Export Drops Large Files

## Why

`export_castore_to_disk()` silently produces empty files or empty directories
when exporting large build outputs. The self-build proved this: crunch compiles
itself successfully inside the bwrap sandbox (exit 0, PathInfo persisted to
redb, NAR hash computed), but the exported output at `--store /tmp/crunch-store`
contains `bin/` with zero entries. The ~30MB crunch binary exists in the
in-memory castore blob service but never reaches the filesystem.

Smaller outputs (selftest, dash, make) export fine. The failure correlates
with blob size but the exact threshold is unknown.

This blocks the self-build from producing a usable binary on disk. The build
is correct — rerunning with the same drv hash would hit the castore cache and
report "cached" — but the user gets an empty output directory.

## What Changes

1. **Diagnose**: instrument `export_castore_to_disk()` to log blob sizes and
   any I/O errors during file writes. Currently errors are silently swallowed
   (`Ok(())` on permission denied, `?` on other I/O but no logging).

2. **Fix the write path**: the likely cause is one of:
   - Blob chunking: large blobs may be stored as multiple chunks in the
     `BlobService`. The export reads via `open_read()` which returns a stream.
     If the stream isn't fully consumed (early drop, async cancellation),
     the file is created but truncated to zero.
   - Async/sync mismatch: `export_castore_to_disk` is called from sync
     context via `block_on`. If the blob read future is dropped before
     completion, the file ends up empty.
   - FUSE read path: if the output was originally read from a FUSE mount
     and the mount is torn down before export completes, reads fail silently.

3. **Add size verification**: after writing each file, compare bytes written
   against the blob's declared size. Fail loudly on mismatch.

4. **Test with the self-build**: the crunch self-build (`bootstrap/crunch.ncl`)
   is the regression test. A successful build should produce a runnable binary
   at `$store/...-crunch/bin/crunch`.

## Capabilities

### Modified Capabilities
- `castore-export`: export must handle blobs of any size, not just small ones
- `build-output`: root build outputs must be fully materialized on disk

## Impact

- **Files**: `crates/crunch-build/src/export.rs`, possibly `orchestrate.rs`
- **APIs**: no public API changes
- **Dependencies**: none
- **Testing**: self-build integration test; unit test with a large synthetic blob
