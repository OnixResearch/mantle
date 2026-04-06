# Tasks: Castore Export Large Files

## Diagnosis

- [ ] Add `tracing::debug!` to `export_castore_to_disk` showing blob digest,
  declared size, and bytes_written for each file export
- [ ] Reproduce with self-build: run `crunch build bootstrap/crunch.ncl`,
  check if `bin/crunch` is zero-length vs missing vs correct size
- [ ] Check if `open_read()` returns the full blob or truncates on chunked
  blobs — add a standalone test reading a >1MB chunked blob

## Fix

- [ ] Add byte-count verification after `tokio::io::copy` in
  `export_castore_to_disk` — return `Error::ExportFailed` on mismatch
- [ ] Audit `persist_and_export_output` error handling — ensure root output
  export errors propagate to the caller (not swallowed)
- [ ] Fix the actual data loss bug (depends on diagnosis — likely chunked
  blob reassembly or async drop issue)

## Testing

- [ ] Unit test: export a 10MB synthetic blob, verify file size matches
- [ ] Unit test: export a chunked blob (multiple chunks via fastcdc), verify
  reassembly produces correct content
- [ ] Integration test: self-build produces a non-empty executable at
  `$store/.../bin/crunch`
