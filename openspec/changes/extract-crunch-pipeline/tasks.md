# Tasks: Extract crunch-pipeline

## Phase 1: Scaffold crate, move types

- [ ] Create `crates/crunch-pipeline/` with `Cargo.toml`, `src/lib.rs`
- [ ] Add `crunch-pipeline` to workspace members in root `Cargo.toml`
- [ ] Define `BuildConfig`, `PipelineResult`, `FodMismatch`, `Error` types
- [ ] Add deps: crunch-eval, crunch-glue, crunch-build, crunch-store, nix-compat, tokio, tracing
- [ ] `cargo check` passes with the new crate (types only, no logic)

## Phase 2: Extract pipeline functions

- [ ] Move `deserialize_derivations_from_json()` to crunch-pipeline
- [ ] Move `resolve_max_jobs()` to crunch-pipeline
- [ ] Move `parse_fod_mismatch_error()` to crunch-pipeline
- [ ] Move the convert loop + registry bridging from `execute_builds_streaming` into a `convert_all()` function
- [ ] Move store/builder/worker setup into `build()` async fn
- [ ] Move result collection (outcome iteration, FOD extraction) into `build()`
- [ ] Delete `execute_builds` (dead-code non-streaming path)
- [ ] `cargo check` passes

## Phase 3: Rewire main.rs

- [ ] Add `crunch-pipeline` dep to the binary's `Cargo.toml`
- [ ] Remove direct deps on crunch-eval, crunch-glue, crunch-build that are no longer used from main.rs (keep crunch-store for `cmd_store`)
- [ ] Rewrite `cmd_build` to construct `BuildConfig` and call `crunch_pipeline::build()`
- [ ] Move `handle_fod_mismatch` + `auto_fix_hash` to a local `fix.rs` module in the binary
- [ ] Keep `write_log`, `log_dir`, `state_dir`, `build_import_paths` in the binary (CLI concerns)
- [ ] Rewrite `cmd_self_build` to delegate to `crunch_pipeline::build()`
- [ ] `cargo check` passes, `main.rs` under 400 lines

## Phase 4: Tests

- [ ] Unit tests for `deserialize_derivations_from_json` (single drv, package set, invalid JSON)
- [ ] Unit test for `resolve_max_jobs` (clamping, default)
- [ ] Unit test for `parse_fod_mismatch_error` (valid, invalid, edge cases)
- [ ] Integration test: pipeline builds a trivial .ncl derivation end-to-end
- [ ] `cargo test -p crunch-pipeline` passes
- [ ] `cargo test --workspace` passes (no regressions)
