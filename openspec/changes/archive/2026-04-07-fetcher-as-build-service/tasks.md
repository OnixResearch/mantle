## Phase 1: FetchBuildService

- [x] Create `fetch_build_service.rs` in crunch-build ✅ 25m (started: 2026-04-07T12:00Z → completed: 2026-04-07T12:25Z)
- [x] Implement `FetchBuildService<BS, DS>` with blob/directory service fields ✅ (in fetch_build_service.rs)
- [x] Implement `BuildService::do_build()` — decode fetch requests from `BuildRequest.command_args[0]` and `environment_vars`, download or extract, and produce a `BuildResult` with output Nodes ✅ (in fetch_build_service.rs)
- [x] Move download/extraction logic from `fetcher.rs::fetch_to_store()` to work with castore Nodes (not disk paths) ✅ Made fetch_flat and fetch_git pub(crate); FetchBuildService downloads to tempdir → ingest_path → Node
- [x] Unit tests: fetch request parsing, fetch URL → BuildResult, fetch tarball → BuildResult, invalid request rejected ✅ 17 tests pass

## Phase 2: DispatchBuildService

- [x] Create `dispatch_build_service.rs` in crunch-build ✅ 5m (started: 2026-04-07T12:26Z → completed: 2026-04-07T12:31Z)
- [x] Implement `DispatchBuildService<F, S>` wrapping fetch + sandbox services ✅ (in dispatch_build_service.rs)
- [x] Route based on `request.command_args.first()`: `builtin:fetchurl` → fetch, everything else → sandbox ✅
- [x] Unit tests: dispatch routes correctly and never sends fetch requests to the sandbox service ✅ 6 tests pass

## Phase 3: Remove fetcher special case from Builder

- [x] Remove `is_builtin_fetcher()` check from `prepare_build()` ✅ 10m (started: 2026-04-07T12:32Z → completed: 2026-04-07T12:42Z)
- [x] Remove `build_fetcher()` method from Builder ✅ (deleted from orchestrate.rs)
- [x] Ensure `derivation_to_build_request()` preserves the fetch builder selector and fetch env vars in `BuildRequest` ✅ 6 tests in build_request.rs: fetcher_build_request_has_builtin_builder, fetcher_build_request_preserves_url_env_var, fetcher_build_request_preserves_all_fetch_env_vars (url+unpack+type+rev+executable), fetcher_build_request_is_recognized_by_is_fetch_request, sandbox_build_request_is_not_fetch_request, fetcher_build_request_outputs_contain_fetch_output_path
- [x] Move all FOD verification and mismatch cleanup into `finish_build()` for both flat and recursive fetchers ✅ 4 tests through DispatchBuildService(FetchBuildService, PanicSandboxService): flat match (PathInfo+CA persisted), flat mismatch (FOD hash mismatch error with expected/actual SRI, no PathInfo, no output_nodes), recursive match (tarball unpack, NAR CA hash verified+persisted), recursive mismatch (NAR FOD mismatch error, no PathInfo, no output_nodes)
- [x] All fetcher derivations go through normal prepare → dispatch → finish path ✅ 257 crunch-build tests pass (PanicSandboxService proves sandbox never called for fetchers)

## Phase 4: Wire up in main.rs / pipeline

- [x] Construct `FetchBuildService` with blob/directory services ✅ 5m (started: 2026-04-07T12:44Z → completed: 2026-04-07T12:49Z)
- [x] Construct `DispatchBuildService` wrapping fetch + bwrap services ✅ (crunch-pipeline/src/lib.rs, src/bootstrap.rs)
- [x] Pass `DispatchBuildService` to Builder instead of raw `BubblewrapBuildService` ✅
- [x] Verify all fetcher integration tests pass ✅ crunch-build: 257, crunch-pipeline: 13 (incl. pipeline_reports_fod_mismatch exercising full fetchurl path), crunch-store: 24
- [x] Verify bootstrap --fetch still works ✅ `crunch bootstrap --fetch --store /tmp/crunch-test-fetch` fetched musl-gcc through DispatchBuildService → FetchBuildService
