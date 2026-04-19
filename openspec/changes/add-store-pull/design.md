## Context

The push module (`crunch-store/src/push.rs`) writes a flat Nix binary cache
layout: `<hash>.narinfo` files plus `nar/<nar-hash>.nar` archives. Pull is the
symmetric read path: scan narinfos, resolve NARs, ingest into castore, persist
PathInfo.

The remote substitution path in `NixHTTPPathInfoService` already does this for
HTTP: fetch narinfo text → parse → download NAR → `ingest_nar_and_hash()` →
construct PathInfo → `put()`. Pull reuses the same primitives but reads from
the filesystem instead of HTTP.

## Goals / Non-Goals

**Goals:**
- Import any narinfo+NAR pair from a flat cache directory into the local store.
- Verify narinfo signatures against supplied trusted public keys before import.
- Verify NAR hash matches narinfo `NarHash` during ingestion.
- Skip paths already present in the local PathInfo database (idempotent).
- Report results: imported count, skipped (present/untrusted/corrupt), bytes.
- Handle store prefix mismatches: if the narinfo `StorePath` uses a different
  prefix than the local store, reject the path with a clear error.

**Non-Goals:**
- HTTP pull (that's existing substitution via `--substituters`).
- Decompression (xz, zstd). Phase 1 handles uncompressed NARs only — matching
  the push output. Compression support is a follow-up for both push and pull.
- Importing narinfos without corresponding NAR files. If the NAR is missing,
  the path is skipped with an error in the report.
- Closure walking during pull. The caller imports explicit paths; transitive
  closure import is a higher-level operation.

## Decisions

### 1. Scan narinfo files, not a manifest

**Choice:** `import_paths_from_cache_dir()` reads `*.narinfo` from the source
directory root. No separate manifest or index file.

**Rationale:** This matches the standard Nix cache layout. The narinfo file IS
the index entry. Scanning `*.narinfo` is O(n) in paths, which is acceptable for
up to `MAX_PULL_PATHS` (1,000,000 — same limit as push).

### 2. NAR path resolution from narinfo URL field

**Choice:** Resolve the NAR file by joining the source directory with the
narinfo `URL` field (typically `nar/<hash>.nar`).

**Rationale:** The URL field is the canonical reference. It works for both
standard (`nar/<hash>.nar`) and non-standard layouts. If the file doesn't
exist at that path, skip with an error.

### 3. Signature verification before ingestion

**Choice:** Parse and verify narinfo signatures against trusted public keys
BEFORE ingesting the NAR. If no signature matches any trusted key and
`trust_unsigned` is false, skip the path.

**Rationale:** NAR ingestion is expensive (reads and hashes the full archive).
Rejecting untrusted narinfos before I/O is cheaper. This matches the remote
substitution path which verifies narinfo trust before downloading the NAR.

### 4. NAR hash verification during ingestion

**Choice:** Use `ingest_nar_and_hash()` which computes the NAR hash during
ingestion and compares it against the narinfo's `NarHash`. If mismatched,
the ingested castore content is orphaned (GC will clean it) and the path
is reported as corrupt.

**Rationale:** This is exactly what the remote substitution path does. The
hash verification is not optional — a cache directory could be corrupt or
tampered with.

### 5. PathInfo construction from narinfo fields

**Choice:** Construct `PathInfo` from parsed narinfo fields: `store_path`,
`nar_sha256`, `nar_size`, `references`, `signatures`, `ca`, `deriver`.

**Rationale:** `NarInfo` carries all the fields needed for a `PathInfo`.
The remote substitution path constructs PathInfo the same way. We reuse
the same field mapping.

### 6. Persist via `pathinfo_service.put()`, not `persist_and_export_signed_output()`

**Choice:** Use `pathinfo_service.put()` directly for imported paths, then
export the castore node to disk separately.

**Rationale:** `persist_and_export_signed_output()` requires `output_name`
(derivation output label) which narinfos don't carry. It also records
provenance attestations, which don't apply to imported paths (provenance
comes from the builder, not the importer). Direct `put()` + export keeps
the import path simple. The attestation gap is acceptable: imported paths
have narinfo signatures as their trust anchor, not build provenance.

### 7. Pull report mirrors push report

**Choice:** `PullReport` has symmetric structure to `PushReport`:

```rust
pub struct PullReport {
    pub imported_count: u32,
    pub skipped_already_present_count: u32,
    pub skipped_untrusted_count: u32,
    pub skipped_hash_mismatch_count: u32,
    pub skipped_missing_nar_count: u32,
    pub skipped_parse_error_count: u32,
    pub total_nar_bytes: u64,
    pub paths: Vec<PulledPath>,
}
```

**Rationale:** Consistent reporting across push/pull. The CLI formats both
the same way.

### 8. Store prefix validation

**Choice:** Before importing, compare the narinfo `StorePath` prefix with
the local `handle.store_dir()`. If they differ, skip the path with a
`skipped_store_dir_mismatch` report entry.

**Rationale:** A `/nix/store` narinfo imported into a `/crunch/store`
local store would produce a PathInfo whose store path doesn't match the
local namespace. This would break closure resolution and output export.
Reject early with a clear message.

## Risks / Trade-offs

**[Uncompressed NARs only]** Phase 1 cannot import compressed NARs (xz,
zstd). Cache directories produced by Nix's `nix copy --to file://` use
xz compression. Mitigation: crunch's own push produces uncompressed NARs,
so the push→pull round-trip works. Cross-tool import from Nix-produced
caches requires compression support (follow-up).

**[Orphaned castore on hash mismatch]** If `ingest_nar_and_hash()` succeeds
but the hash check fails, blobs and directories remain in castore until GC.
This is the same behavior as the remote substitution path and is acceptable.

**[No closure import]** Importing a single path without its runtime closure
means the path may not be usable for builds until its references are also
imported. Mitigation: `--all` imports everything; selective import requires
the operator to know the closure. A `--with-closure` flag is a natural
follow-up.
