## Phase 1: Wire RedbPathInfoService into CLI

- [ ] Add `state_dir()` helper in `src/main.rs` (same logic as `log_dir()` but returns parent: `$CRUNCH_STATE_DIR` or `$XDG_STATE_HOME/crunch` or `$HOME/.local/state/crunch`)
- [ ] Instantiate `RedbPathInfoService` in `cmd_build()` with `path = state_dir/pathinfo.redb`, `read_only = false`
- [ ] Handle database open failure: log warning, fall back to `LruPathInfoService` (in-memory)
- [ ] Test: first run creates the redb file at the expected location

## Phase 2: Builder accepts PathInfoService

- [ ] Add `PIS: PathInfoService` type parameter to `Builder<BS, DS, BServ, PIS>`
- [ ] Store `pathinfo_service: PIS` in Builder
- [ ] After successful build in `build_derivation_inner`, call `pathinfo_service.put(path_info)` for each output
- [ ] Verify put is called by adding a counting wrapper in unit tests
- [ ] Test: after build, `pathinfo_service.get(digest)` returns the stored PathInfo

## Phase 3: Cache check via PathInfoService

- [ ] Change `all_outputs_exist()` to `check_cache()` — for each output, call `pathinfo_service.get(digest)`. Cache hit = `Some(path_info)` AND `path_exists_on_disk(path)`
- [ ] On cache hit, return stored PathInfo directly (no `ingest_path`, no `calculate_nar`)
- [ ] Extract `Node` from stored PathInfo and insert into `output_nodes` (for downstream builds)
- [ ] Remove `load_cached_outputs()` method (replaced by PathInfo lookup)
- [ ] Log warning when PathInfo exists but file doesn't (inconsistent store)
- [ ] Test: cache hit returns stored PathInfo without calling `ingest_path`
- [ ] Test: PathInfo exists but file deleted = cache miss, rebuilds
- [ ] Test: file exists but no PathInfo = cache miss, rebuilds

## Phase 4: crunch store subcommand

- [ ] Add `Command::Store` variant with sub-subcommands: `List`, `Info { path: String }`, `Verify { path: Option<String> }`
- [ ] `crunch store list`: open redb read-only, iterate `PathInfoService::list()`, print store_path, deriver name, nar_size
- [ ] `crunch store info <path>`: parse store path digest, call `get()`, print all PathInfo fields
- [ ] `crunch store verify [path]`: re-ingest from disk, compute NAR hash, compare with stored. Report match/mismatch.
- [ ] Test: `crunch store list` after a build shows the built path
- [ ] Test: `crunch store verify` detects a modified file

## Phase 5: Integration and cleanup

- [ ] Integration test: build, exit, re-run — second run is a cache hit (no rebuild)
- [ ] Integration test: build, delete output file, re-run — rebuilds and re-records
- [ ] Integration test: `CRUNCH_STATE_DIR` override works
- [ ] Update mock tests: `Builder::new()` gains PathInfoService parameter, use `LruPathInfoService` in existing tests
- [ ] Update README: mention persistent cache, `crunch store` subcommand
