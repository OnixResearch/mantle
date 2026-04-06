# Design: Castore Export Large Files

## Context

`export_castore_to_disk()` in `export.rs` walks a `Node` tree and writes
files/dirs/symlinks to the output directory. File contents come from
`blob_service.open_read(digest)` which returns an async `BlobReader` stream.

The current code:
```rust
let mut reader = blob_service.open_read(&digest).await?
    .ok_or(...)?;
let mut file = tokio::fs::File::create(&path).await?;
tokio::io::copy(&mut reader, &mut file).await?;
```

For large blobs (>1MB), the blob service may store data in multiple chunks
(fastcdc chunking in `snix-castore`). The `open_read()` assembles chunks
back into a contiguous stream. If this stream is interrupted or the async
runtime context is wrong, zero bytes are written.

## Goals

- Large file exports work reliably (tested up to 100MB).
- Failures are loud: error returned, not silently swallowed.
- No performance regression for small files.

## Non-Goals

- Streaming export during build (current: export after build completes).
- Parallel file writes within a single output tree.

## Approach

### 1. Add byte-count verification

After `tokio::io::copy`, check that bytes_written matches the blob's
declared size (from the `FileNode`). If mismatch, return
`Error::ExportFailed` with sizes and path.

### 2. Audit error handling in export path

The `persist_and_export_output` → `export_castore_to_disk` call chain
currently has:
- `export_castore_to_disk` returns `Result<()>` but callers in
  `orchestrate.rs` may ignore errors for non-root outputs
- For root outputs, errors should propagate and fail the build

Trace every `?` and `.ok()` in the chain. Replace any silent error
suppression with explicit logging.

### 3. Test with synthetic large blob

Create a test that:
1. Writes a 10MB blob to `MemoryBlobService`
2. Creates a `FileNode` referencing it
3. Calls `export_castore_to_disk`
4. Verifies the output file has the correct size and content

### 4. Integration test: self-build binary

The self-build (`bootstrap/crunch.ncl`) produces a ~30MB binary. After
fixing the export, verify that `$store/.../bin/crunch` exists and is
executable.

## Risks

- The bug might be in the blob service's chunked reassembly, not in
  the export code. If so, the fix is deeper (in snix-castore).
- The async runtime context matters: `export_castore_to_disk` runs inside
  `tokio::spawn` from the worker. If the task is cancelled (e.g., on
  partial failure), in-progress exports get dropped.
