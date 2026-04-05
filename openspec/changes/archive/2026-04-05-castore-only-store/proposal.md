## Why

crunch requires a writable filesystem store to cache builds. `check_cache`
demands both a PathInfo record AND the output existing on disk at
`output_dir`. On a read-only `/nix/store` (the common NixOS case), every
build is a cache miss — PathInfo is persisted in redb, but the disk
export silently fails, so the next run sees "PathInfo exists but output
missing from disk" and rebuilds.

The workaround is an overlay mount via `unshare`. That's fine for testing
but wrong for normal use. The castore already holds every build output as
content-addressed blobs and directory nodes. The build pipeline between
derivations already flows through `output_nodes` (castore Nodes), not
disk paths. The disk export is only needed for the user's final outputs.

## What Changes

- **Cache validation**: `check_cache` trusts PathInfo + castore content
  instead of PathInfo + disk existence. Probe `blob_service.has(digest)`
  for the root blob/directory of each output's `Node` rather than
  `PathBuf::exists()`.
- **Disk export scope**: only export to disk for top-level roots
  requested by the user, not intermediate dependencies. Intermediate
  deps stay in castore only.
- **`output_dir` becomes optional**: when unset (or set to a
  non-writable path), crunch still builds and caches correctly. Disk
  export is best-effort for roots only, with a clear warning when
  skipped.
- **Source input ingestion unchanged**: seed packages are still read
  from `/nix/store/` (read-only access) and ingested into castore on
  first use. This already works.

## Capabilities

### New Capabilities
- `castore-cache`: builds are cached in the castore without requiring
  a writable filesystem store.
- `root-only-export`: disk materialization happens only for the
  derivations the user explicitly requested.

### Modified Capabilities
- `check-cache`: no longer requires filesystem existence.
- `persist-output`: intermediate deps skip disk export entirely.

## Impact

- **Files**: `crates/crunch-build/src/orchestrate.rs` (check_cache,
  persist_and_export_output, build_derivation_inner).
- **APIs**: `Builder::new` signature unchanged. `output_dir` becomes
  semantically "where to export final roots" rather than "the store."
- **Dependencies**: none.
- **Testing**: update cache tests that assert disk existence. Add test
  for cache hit with PathInfo + castore but no file on disk. Add test
  for root-only export behavior.
