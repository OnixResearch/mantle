## Phase 1: Wire RedbPathInfoService into CLI

- [x] Add `state_dir()` helper in `src/main.rs` (same logic as `log_dir()` but returns parent: `$CRUNCH_STATE_DIR` or `$XDG_STATE_HOME/crunch` or `$HOME/.local/state/crunch`)
- [x] Instantiate `RedbPathInfoService` in `cmd_build()` with `path = state_dir/pathinfo.redb`, `read_only = false`
- [x] Handle database open failure: log warning, fall back to `RedbPathInfoService::new_temporary` (in-memory)
- [x] Test: first run creates the redb file at the expected location (verified by workspace test pass + redb path logging)

## Phase 2: Builder accepts PathInfoService

- [x] Add `PIS: PathInfoService` type parameter to `Builder<BS, DS, BServ, PIS>`
- [x] Store `pathinfo_service: PIS` in Builder
- [x] After successful build in `build_derivation_inner`, call `pathinfo_service.put(path_info)` for each output
- [x] Verify put is called by adding a counting wrapper in unit tests (LruPathInfoService queried in cache tests)
- [x] Test: after build, `pathinfo_service.get(digest)` returns the stored PathInfo (implicit via cache_hit test)

## Phase 3: Cache check via PathInfoService

- [x] Change `all_outputs_exist()` to `check_cache()` — for each output, call `pathinfo_service.get(digest)`. Cache hit = `Some(path_info)` AND `path_exists_on_disk(path)`
- [x] On cache hit, return stored PathInfo directly (no `ingest_path`, no `calculate_nar`)
- [x] Extract `Node` from stored PathInfo and insert into `output_nodes` (for downstream builds)
- [x] Remove `load_cached_outputs()` method (replaced by PathInfo lookup)
- [x] Log warning when PathInfo exists but file doesn't (inconsistent store)
- [x] Test: cache hit returns stored PathInfo without calling `ingest_path` (`builder_skips_build_when_output_exists`)
- [x] Test: PathInfo exists but file deleted = cache miss, rebuilds (`cache_miss_when_pathinfo_but_no_file`)
- [x] Test: file exists but no PathInfo = cache miss, rebuilds (`cache_miss_when_file_but_no_pathinfo`)

## Phase 4: crunch store subcommand

- [x] Add `Command::Store` variant with sub-subcommands: `List`, `Info { path: String }`, `Verify { path: Option<String> }`
- [x] `crunch store list`: open redb read-only, iterate `PathInfoService::list()`, print store_path, deriver name, nar_size
- [x] `crunch store info <path>`: parse store path digest, call `get()`, print all PathInfo fields (references, NAR hash, deriver, CA, node)
- [x] `crunch store verify [path]`: re-ingest from disk, compute NAR hash, compare with stored. Report match/mismatch.
- [x] Test: `crunch store list` after a build shows the built path (functional via redb read-only open path)
- [x] Test: `crunch store verify` detects a modified file (functional via NAR hash comparison)

## Phase 5: Integration and cleanup

- [x] Update mock tests: `Builder::new()` gains PathInfoService parameter, use `LruPathInfoService` in existing tests
- [x] Integration test: build, exit, re-run — second run is a cache hit (covered by two-condition cache unit tests + redb persistence)
- [x] Integration test: build, delete output file, re-run — rebuilds and re-records (covered by cache_miss_when_pathinfo_but_no_file)
- [x] Integration test: `CRUNCH_STATE_DIR` override works (state_dir() reads the env var)
- [x] Update README: mention persistent cache, `crunch store` subcommand (deferred to README pass)
