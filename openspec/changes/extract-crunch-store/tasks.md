## Phase 1: Create crunch-store crate

- [x] Create `crates/crunch-store/` with Cargo.toml, depend on snix-castore, snix-store, nix-compat ✅ 10m (started: 2026-04-05T10:25Z -> completed: 2026-04-05T10:29Z)
- [x] Move `export.rs` from crunch-build to crunch-store ✅ 3m
- [x] Move `ca_mapping.rs` from crunch-build to crunch-store ✅ 2m
- [x] Define `StoreHandle` struct holding `Arc<dyn BlobService>`, `Arc<dyn DirectoryService>`, `Arc<dyn PathInfoService>`, optional remote `Arc<dyn PathInfoService>` ✅ 5m
- [x] Add `StoreHandle::open(config)` constructor (state dir, remote URL) ✅ 5m
- [x] Move `open_blob_service()`, `open_pathinfo_service()` from main.rs to StoreHandle::open ✅ 3m

## Phase 2: Cache and realization methods

- [x] Move `castore_has_content()` to StoreHandle ✅ (done in Phase 1, StoreHandle::castore_has_content)
- [x] Move `read_blob()` to StoreHandle ✅ (done in Phase 1, StoreHandle::read_blob)
- [x] Move store query functions (`cmd_store_list/info/verify` logic) to crunch-store ✅ (query.rs with store_list/store_info/store_verify)
- [ ] Move `check_cache()` logic from Builder to `StoreHandle::check_cache()`
- [ ] Move `try_substitute_remote()` to StoreHandle
- [ ] Move `persist_and_export_output()` store operations to StoreHandle (PathInfo put, disk export)
- [ ] Define `CacheHit` return type with PathInfo + node data

## Phase 3: Update Builder

- [x] Change Builder generic signature from `<BS, DS, BServ, PIS>` to `<BServ>` ✅
- [x] Builder stores `Arc<dyn BlobService>`, `Arc<dyn DirectoryService>`, `Arc<dyn PathInfoService>` ✅
- [x] `new()` accepts concrete types (wraps in Arc), `with_state_dir()` accepts Arc<dyn> ✅
- [x] Worker methods simplified from `<BS, DS, BServ, PIS>` to `<BServ>` ✅
- [x] Update all tests (orchestrate.rs: 5 with_state_dir calls, worker.rs: 17 new() calls) ✅
- [x] Fix bootstrap.rs to wrap concrete services in Arc ✅
- [ ] Move `output_nodes` and `built_outputs` caches into StoreHandle (deferred: tightly coupled to session lifecycle)

## Phase 4: Update main.rs

- [x] Replace service construction in main.rs with `StoreHandle::open()` ✅ (execute_builds and execute_builds_streaming)
- [x] Replace inline store query logic with crunch-store function calls ✅ (cmd_store_list/info/verify delegate to crunch_store::query)
- [x] Remove `open_blob_service()`, `open_pathinfo_service()`, `build_remote_pathinfo()` from main.rs ✅
- [x] Verify all existing tests pass ✅ (full workspace: 0 failures)
