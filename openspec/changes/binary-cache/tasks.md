## Phase 1: Builder Plumbing

- [x] Add `remote_pathinfo: Option<Arc<dyn PathInfoService>>` field to `Builder` ✅ 5m
- [x] Update `Builder::new` and `Builder::with_state_dir` to accept the new param (default `None`) ✅ 5m
- [x] Update all existing callers (main.rs `cmd_build`, `cmd_store`, tests) to pass `None` ✅ 5m
- [x] Verify: workspace compiles, all existing tests pass unchanged ✅ 2m

## Phase 2: Remote Fallback in check_cache

- [x] After local miss in `check_cache`, query `remote_pathinfo.get(digest)` if available ✅ 10m
- [x] On remote hit: persist to local `pathinfo_service.put()`, cache node in `output_nodes`, return as cached ✅ 5m
- [x] Skip remote query for FODs (any output has `ca_hash`) and CA derivations (output.path is None) ✅ 5m
- [x] Wrap remote errors in `tracing::warn!` and treat as miss (no build failure) ✅ 3m
- [x] Unit test: mock remote returns PathInfo -> check_cache returns cached ✅ 5m
- [x] Unit test: mock remote returns None -> check_cache returns None (build proceeds) ✅ 3m
- [x] Unit test: FOD derivation -> remote not queried ✅ 5m
- [x] Unit test: remote error -> warning logged, treated as miss ✅ 5m

## Phase 3: CLI Flags + NixHTTPPathInfoService Construction

- [x] Add `--substituters <url>` flag to `build` subcommand (default: `https://cache.nixos.org`) ✅ 5m
- [x] Add `--no-substitute` flag to `build` subcommand ✅ 2m
- [x] In `cmd_build`: construct `NixHTTPPathInfoService` from substituter URL, sharing blob/dir services ✅ 10m
- [x] Pass `Some(Arc::new(remote_pis))` to Builder when substitution is enabled ✅ 3m
- [x] Unit test: write-through verified (remote hit persisted to local, second build instant) ✅ 5m

## Phase 4: End-to-End Verification

- [ ] Manual test: `crunch build` a derivation whose output exists on cache.nixos.org, verify substitution
- [ ] Verify: substituted path appears in local redb, second build is instant (no network)
- [x] Add CLI output: "substituting /nix/store/...-hello from https://cache.nixos.org" ✅ 2m (via tracing::info in try_substitute_remote)
