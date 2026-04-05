## Phase 1: Castore Cache Validation

- [x] Add `castore_has_content(&self, node: &Node) -> Result<bool, Error>` method to Builder — probes blob_service/directory_service
- [x] Rewrite `check_cache` to use `castore_has_content` instead of `PathBuf::exists()`
- [x] Populate `output_nodes` from PathInfo.node on cache hit
- [x] Remove the `(Some(_), false)` "PathInfo but no file" rebuild branch
- [x] Unit test: cache hit when PathInfo + castore content present, no file on disk
- [x] Unit test: cache miss when PathInfo present but castore content missing
- [x] Update existing cache tests that assert disk existence behavior

## Phase 2: Root-Only Export

- [x] Thread root set via `is_root` on PreparedBuild, through prepare_build/finish_build/process_output/persist_and_export_output
- [x] Skip `export_castore_to_disk` for non-root builds in `persist_and_export_output`
- [x] Emit warning (not error) when root export fails due to read-only store
- [x] Integration test: build with dep chain, verify only root output on disk
- [x] Integration test: build with unwritable output_dir, verify success + warning

## Phase 3: Cleanup

- [x] Update README "Known Limitations" section — read-only /nix/store now works
- [x] Remove overlay mount workaround from docs (none to remove)
- [x] Update `--store` docs: "where to export final outputs" not "the store"
