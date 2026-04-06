## Phase 1: Create crunch-store crate

- [ ] Create `crates/crunch-store/` with Cargo.toml, depend on snix-castore, snix-store, nix-compat
- [ ] Move `export.rs` from crunch-build to crunch-store
- [ ] Move `ca_mapping.rs` from crunch-build to crunch-store
- [ ] Define `StoreHandle` struct holding `Arc<dyn BlobService>`, `Arc<dyn DirectoryService>`, `Arc<dyn PathInfoService>`, optional remote `Arc<dyn PathInfoService>`
- [ ] Add `StoreHandle::open(config)` constructor (state dir, remote URL)
- [ ] Move `open_blob_service()`, `open_pathinfo_service()` from main.rs to StoreHandle::open

## Phase 2: Cache and realization methods

- [ ] Move `check_cache()` logic from Builder to `StoreHandle::check_cache()`
- [ ] Move `try_substitute_remote()` to StoreHandle
- [ ] Move `castore_has_content()` to StoreHandle
- [ ] Move `persist_and_export_output()` store operations to StoreHandle (PathInfo put, disk export)
- [ ] Define `CacheHit` return type with PathInfo + node data
- [ ] Move store query functions (`cmd_store_list/info/verify` logic) to crunch-store

## Phase 3: Update Builder

- [ ] Change Builder generic signature from `<BS, DS, BServ, PIS>` to `<BServ>`
- [ ] Replace direct service calls in Builder with StoreHandle method calls
- [ ] Move `output_nodes` and `built_outputs` caches into StoreHandle (session-scoped)
- [ ] Update all tests in orchestrate.rs to construct StoreHandle

## Phase 4: Update main.rs

- [ ] Replace service construction in main.rs with `StoreHandle::open()`
- [ ] Replace inline store query logic with crunch-store function calls
- [ ] Verify all existing tests pass
