## Why

`KnownPaths::get_by_drv_path()` does a linear scan over all entries in
`by_aterm_hash` on every call. The build orchestrator calls this for every
derivation it builds. For small dependency graphs this is fine, but as
crunch handles real package sets the scan becomes O(n) per derivation,
O(n²) overall.

The fix is a one-line index: a `HashMap<String, [u8; 32]>` mapping drv
path strings to their aterm hash, maintained alongside `hdm_by_drv_path`
in `insert()`.

## What Changes

- Add a `by_drv_path` index to `KnownPaths`.
- Replace the linear scan in `get_by_drv_path()` with a two-step lookup:
  drv_path → aterm_hash → KnownEntry.
- Add direct unit tests for `KnownPaths`.

## Capabilities

### Modified Capabilities
- `known-paths-lookup`: O(1) lookup by drv path instead of O(n) scan.

## Impact

- **Files**: `crates/crunch-glue/src/known_paths.rs`
- **APIs**: No public API change (same signature, faster)
- **Dependencies**: None
- **Testing**: Unit tests in known_paths.rs
