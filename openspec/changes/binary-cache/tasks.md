## Phase 1: Builder Plumbing

- [ ] Add `remote_pathinfo: Option<Arc<dyn PathInfoService>>` field to `Builder`
- [ ] Update `Builder::new` and `Builder::with_state_dir` to accept the new param (default `None`)
- [ ] Update all existing callers (main.rs `cmd_build`, `cmd_store`, tests) to pass `None`
- [ ] Verify: workspace compiles, all existing tests pass unchanged

## Phase 2: Remote Fallback in check_cache

- [ ] After local miss in `check_cache`, query `remote_pathinfo.get(digest)` if available
- [ ] On remote hit: persist to local `pathinfo_service.put()`, cache node in `output_nodes`, return as cached
- [ ] Skip remote query for FODs (any output has `ca_hash`) and CA derivations (output.path is None)
- [ ] Wrap remote errors in `tracing::warn!` and treat as miss (no build failure)
- [ ] Unit test: mock remote returns PathInfo → check_cache returns cached
- [ ] Unit test: mock remote returns None → check_cache returns None (build proceeds)
- [ ] Unit test: FOD derivation → remote not queried
- [ ] Unit test: remote error → warning logged, treated as miss

## Phase 3: CLI Flags + NixHTTPPathInfoService Construction

- [ ] Add `--substituters <url>` flag to `build` subcommand (default: `https://cache.nixos.org`)
- [ ] Add `--no-substitute` flag to `build` subcommand
- [ ] In `cmd_build`: construct `NixHTTPPathInfoService` from substituter URL, sharing blob/dir services
- [ ] Pass `Some(Arc::new(remote_pis))` to Builder when substitution is enabled
- [ ] Integration test: build with `--no-substitute` never queries remote

## Phase 4: End-to-End Verification

- [ ] Manual test: `crunch build` a derivation whose output exists on cache.nixos.org, verify substitution
- [ ] Verify: substituted path appears in local redb, second build is instant (no network)
- [ ] Add CLI output: "substituting /nix/store/...-hello from https://cache.nixos.org"
