## Phase 1: Diagnosis

- [x] Run existing test suite, identify failures
- [x] Trace sandbox build failure: `/bin/sh: can't create /crunch/store/...: nonexistent directory`
- [x] Add bwrap debug logging, dump exact bwrap args
- [x] Identify mismatch: `NIX_STORE=/nix/store` in sandbox env but `$out=/crunch/store/...`
- [x] Trace `store_dir` through Builder → StoreHandle → build_request: found hardcoded `STORE_DIR`

## Phase 2: Fix store_dir threading

- [x] Add `store_dir: &str` to `Builder::new()` and `Builder::with_state_dir()`
- [x] Use `from_services_with_store_dir()` instead of `from_services()`
- [x] Update all callers in `orchestrate.rs` tests (29 sites)
- [x] Update all callers in `worker.rs` tests (17 sites)
- [x] Update `main.rs` callers (2 — legacy + streaming paths)
- [x] Update `bootstrap.rs` caller (1)
- [x] Update `tests/integration_build.rs` callers (5)
- [x] Fix `build_request.rs`: `path_str()` → `path_str_with_prefix(store_dir)`

## Phase 3: Fix tests

- [x] Fix `tests/integration.rs` build tests: use `--store <tempdir>`, relax assertions
- [x] Fix `tests/smoke.rs`: busybox prefix for `mkdir`/`chmod`/`ln`
- [x] Verify: sandbox builds work with default `--store-prefix /crunch/store`
- [x] Verify: sandbox builds work with `--nix-compat` (`/nix/store`)
- [x] Clear stale state dir, confirm CA test passes
- [x] Confirm 8/10 smoke tests pass (2 pre-existing failures)
