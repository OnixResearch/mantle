## Phase 1: Add the index

- [x] Add `drv_path_to_aterm: HashMap<String, [u8; 32]>` field to KnownPaths
- [x] Populate it in `insert()` alongside hdm_by_drv_path
- [x] Rewrite `get_by_drv_path()` as two-step lookup via the index
- [x] Initialize the new field in `new()` and `Default`

## Phase 2: Tests

- [x] Test get_by_drv_path returns correct entry (same behavior, faster path)
- [x] Test get_by_drv_path returns None for unknown path
- [x] Test multiple inserts: each drv path resolves independently
- [x] Test overwrite: re-inserting same drv path updates the index
