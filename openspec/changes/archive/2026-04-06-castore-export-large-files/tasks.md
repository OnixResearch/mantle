## Phase 1: Vendored constructor

- [x] Add `pub fn new_local(path: impl AsRef<Path>) -> io::Result<Self>` to `ObjectStoreBlobService` in `vendor/snix-castore/src/blobservice/object_store.rs`. Uses `LocalFileSystem::new_with_prefix`. Default avg_chunk_size 256 KiB. Empty instance_name. Default base_path. ✅ 5m
- [x] Add unit tests: `new_local_roundtrip` (write+read) and `new_local_persists_across_instances` (write in one instance, read in another). Both pass. ✅ 3m
- [x] `cargo check` passes with the new constructor. ✅ 1m

## Phase 2: Wire into production code

- [x] In `src/main.rs`: add `open_blob_service()` helper returning `Arc<ObjectStoreBlobService>`. Replace `MemoryBlobService::default()` in `execute_builds` and `execute_builds_streaming`. Keep `MemoryBlobService` in `cmd_store_verify` (ephemeral verification, no persistence needed). ✅ 5m
- [x] In `src/bootstrap.rs`: same replacement for the bootstrap blob service. Inline construction (no shared helper — bootstrap.rs can't call main.rs functions). ✅ 3m
- [x] `BubblewrapBuildService::new(workdir, blob_service.clone(), ...)` compiles — `Arc<ObjectStoreBlobService>` satisfies `BS: BlobService + Clone`. ✅ verified
- [x] `cargo check` passes. Full test suite passes (0 failures across workspace). ✅ 2m

## Phase 3: Integration test

- [x] Add `persistent_blob_cache_hit_across_builder_instances` test in orchestrate.rs: creates ObjectStoreBlobService + RedbPathInfoService backed by temp dirs, populates blob+pathinfo in first scope, drops all services, creates fresh Builder in second scope with same dirs, verifies cache hit (no do_build call). ✅ 5m
- [x] Full test suite: 221 crunch-build tests pass (was 220), 0 failures. Workspace-wide: 0 failures. ✅ 2m

## Phase 4: Manual smoke test

- [ ] Build a real derivation with `crunch build` (e.g., `examples/hello.ncl`). Verify output files are non-empty on disk. Verify `~/.local/state/crunch/blobs/` contains chunk files.
- [ ] Run the same build again. Verify it's a cache hit (no rebuild). Verify blob dir contents unchanged.
- [ ] Build a large derivation (e.g., dash or make from bootstrap). Verify no OOM, output files are complete.
