# Tasks: Remove host git from fetchGit

## Phase 1: Fetch implementation

- [x] Delete arbitrary host `git` path discovery from `crates/crunch-build/src/fetcher.rs`, including `PATH` scanning and common host-path probes
- [x] Implement a crunch-controlled `fetchGit` materialization path that resolves the requested revision and writes the checkout tree without `.git/`
- [x] Add contract coverage that `crunch.fetchGit { url, rev, hash }` still evaluates and emits `builder = "builtin:fetchurl"`, `system = "builtin"`, `fixed_output.mode = 'recursive`, `env.url == input url`, `env.type == "git"`, and `env.rev == input rev`

## Phase 2: Errors and tests

- [x] Normalize invalid-revision and fetch failure errors around crunch-owned semantics
- [x] Add tests that invalid revisions report `requested revision '<rev>' could not be materialized`, fetch failures stay crunch-owned instead of host-`git` `fatal:` text, and neither path depends on host `git` stderr formatting
- [x] Add tests that `fetchGit` materializes the requested revision and strips `.git/`
- [x] Add tests that `fetchGit` still materializes the requested revision and the same output tree when host `PATH` is empty
- [x] Add tests that fake host `git` binaries with different reported versions do not change the requested-revision output tree, `.git` stripping, or the crunch-owned-vs-`fatal:` error split
- [x] Add non-`file://` transport coverage that exercises the isolated remote fetch path without using host git helpers from `PATH`
- [x] Add tests that the provided `hash` still controls fixed-output acceptance for `fetchGit`, including recursive hash mismatch handling
- [x] Add a deterministic source-inspection test that fails if non-test fetchGit implementation code reintroduces `std::process::Command`, `std::env::var("PATH")`, version probes, `git-upload-pack`, or fixed host `git` probe paths
- [x] Run `cargo test -p crunch-build --lib --tests`, `cargo test -p crunch --test stdlib_tests fetch_git_produces_git_env -- --nocapture`, `CRUNCH_FORCE_EMBEDDED_STDLIB=1 cargo test -p crunch --test stdlib_tests fetch_git_produces_git_env -- --nocapture`, `cargo test -p crunch --test integration_build fetchgit_ -- --nocapture`, and `openspec validate fetchers`
