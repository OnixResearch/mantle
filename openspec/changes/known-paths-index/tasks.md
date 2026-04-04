## Phase 1: Add the index

- [ ] Add `drv_path_to_aterm: HashMap<String, [u8; 32]>` field to KnownPaths
- [ ] Populate it in `insert()` alongside hdm_by_drv_path
- [ ] Rewrite `get_by_drv_path()` as two-step lookup via the index
- [ ] Initialize the new field in `new()` and `Default`

## Phase 2: Tests

- [ ] Test get_by_drv_path returns correct entry (same behavior, faster path)
- [ ] Test get_by_drv_path returns None for unknown path
- [ ] Test multiple inserts: each drv path resolves independently
- [ ] Test overwrite: re-inserting same drv path updates the index
