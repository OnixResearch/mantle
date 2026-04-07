# Tasks: Extract crunch-pipeline

## Phase 1: Scaffold crate, move types

- [x] Create `crates/crunch-pipeline/` with `Cargo.toml`, `src/lib.rs`
- [x] Add `crunch-pipeline` to workspace members in root `Cargo.toml`
- [x] Define `BuildConfig`, `PipelineResult`, `FodMismatch`, `Error` types
- [x] Add deps: crunch-eval, crunch-glue, crunch-build, crunch-store, nix-compat, tokio, tracing
- [x] `cargo check -p crunch-pipeline` passes after scaffolding the new crate

## Phase 2: Extract pipeline functions

- [x] Move `deserialize_derivations_from_json()` to crunch-pipeline
- [x] Move `resolve_max_jobs()` to crunch-pipeline
- [x] Move `parse_fod_mismatch_error()` to crunch-pipeline
- [x] Move the convert loop + registry bridging from `execute_builds_streaming` into a `convert_all()` function
- [x] Move store/builder/worker setup into `build()` async fn
- [x] Move result collection (outcome iteration, FOD extraction) into `build()`
- [x] Delete `execute_builds` (dead-code non-streaming path)
- [x] `cargo check -p crunch-pipeline` passes after extracting the pipeline functions

## Phase 3: Rewire main.rs

- [x] Add `crunch-pipeline` dep to the binary's `Cargo.toml`
- [ ] Audit remaining direct deps in the binary crate and remove only the ones no longer needed after extraction; keep deps that still have live `eval`, `bootstrap`, or `store` call sites
- [x] Rewrite `cmd_build` to construct `BuildConfig` and call `crunch_pipeline::build()`
- [x] Move `handle_fod_mismatch` + `auto_fix_hash` to a local `fix.rs` module in the binary
- [x] Keep `write_log`, `log_dir`, `state_dir`, `build_import_paths` in the binary (CLI concerns)
- [x] Rewrite `cmd_self_build` to delegate to `crunch_pipeline::build()`
- [x] `cargo check -p crunch` passes, `main.rs` under 400 lines

### Dependency audit notes

- `crunch_eval` still has live binary-crate call sites in `src/main.rs` (`eval`) and `src/build_cmd.rs` (`build_import_paths`).
- `crunch_glue` and `crunch_build` still have live binary-crate call sites in `src/bootstrap.rs`.
- `crunch_store` and `snix_store` remain live for `src/store_cmd.rs`.

## Phase 4: Tests

- [x] Unit tests for `deserialize_derivations_from_json` (single drv, package set, invalid JSON)
- [x] Unit test for `resolve_max_jobs` (clamping, default)
- [x] Unit test for `parse_fod_mismatch_error` (valid, invalid, edge cases)
- [x] Integration test: pipeline builds a trivial .ncl derivation end-to-end
- [x] `cargo test -p crunch-pipeline` passes
- [x] `cargo test --workspace` passes (no regressions)

## Verification evidence

Detailed implementation and command evidence lives in `verification.md`.

### Command output excerpts

```text
$ cargo check -p crunch-pipeline
Checking crunch-pipeline v0.1.0 (/home/brittonr/git/crunch/crunch/crates/crunch-pipeline)
Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.34s

$ cargo check -p crunch
Checking crunch v0.1.0 (/home/brittonr/git/crunch/crunch)
Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.09s

$ cargo test -p fuse-backend-rs transport::fusedev::linux_session::tests::test_new_channel
running 1 test
test transport::fusedev::linux_session::tests::test_new_channel ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 88 filtered out; finished in 0.00s

$ cargo test -p snix-castore --doc
Doc-tests snix_castore
running 3 tests
test vendor/snix-castore/src/composition.rs - composition (line 16) ... ok
test vendor/snix-castore/src/composition.rs - composition (line 84) ... ok
test vendor/snix-castore/src/composition.rs - composition (line 52) ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

$ cargo test -p crunch-pipeline
running 10 tests
...
test tests::parse_fod_mismatch_edge_case_preserves_trailing_context ... ok
...
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

running 1 test
test pipeline_builds_trivial_derivation_end_to_end ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

$ cargo test --workspace
...
test transport::fusedev::linux_session::tests::test_new_channel ... ok
...
Doc-tests snix_castore
running 3 tests
test vendor/snix-castore/src/composition.rs - composition (line 16) ... ok
test vendor/snix-castore/src/composition.rs - composition (line 84) ... ok
test vendor/snix-castore/src/composition.rs - composition (line 52) ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
...
WORKSPACE_EXIT=0
```
