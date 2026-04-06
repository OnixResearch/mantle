## Phase 1: FetchBuildService

- [ ] Create `fetch_build_service.rs` in crunch-build
- [ ] Implement `FetchBuildService<BS, DS>` with blob/directory service fields
- [ ] Implement `BuildService::do_build()` — parse env, download, extract, produce BuildResult with output Node
- [ ] Move download/extraction logic from `fetcher.rs::fetch_to_store()` to work with castore Nodes (not disk paths)
- [ ] Unit tests: fetch URL → BuildResult, fetch tarball → BuildResult, invalid builder rejected

## Phase 2: DispatchBuildService

- [ ] Create `dispatch_build_service.rs` in crunch-build
- [ ] Implement `DispatchBuildService<F, S>` wrapping fetch + sandbox services
- [ ] Route based on builder string: `builtin:fetchurl` → fetch, everything else → sandbox
- [ ] Unit tests: dispatch routes correctly

## Phase 3: Remove fetcher special case from Builder

- [ ] Remove `is_builtin_fetcher()` check from `prepare_build()`
- [ ] Remove `build_fetcher()` method from Builder
- [ ] Ensure `derivation_to_build_request()` produces valid BuildRequests for fetcher derivations
- [ ] Move FOD flat-hash verification to `finish_build()` (alongside NAR-based FOD verification)
- [ ] All fetcher derivations go through normal prepare → dispatch → finish path

## Phase 4: Wire up in main.rs / pipeline

- [ ] Construct `FetchBuildService` with blob/directory services
- [ ] Construct `DispatchBuildService` wrapping fetch + bwrap services
- [ ] Pass `DispatchBuildService` to Builder instead of raw `BubblewrapBuildService`
- [ ] Verify all fetcher integration tests pass
- [ ] Verify bootstrap --fetch still works
