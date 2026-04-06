## Phase 1: Create crunch-pipeline crate

- [ ] Create `crates/crunch-pipeline/` with Cargo.toml
- [ ] Define `BuildOpts`, `EvalOpts`, `BootstrapOpts` config structs
- [ ] Define `BuildResult` struct (outcomes, logs, failures)
- [ ] Move `deserialize_derivations_from_json()` from main.rs to crunch-pipeline
- [ ] Move `state_dir()`, `log_dir()` from main.rs to crunch-pipeline
- [ ] Move `write_log()`, log retrieval logic from main.rs to crunch-pipeline

## Phase 2: Pipeline entry points

- [ ] Implement `pipeline::eval(opts)` — wraps crunch-eval::evaluate_to_json
- [ ] Implement `pipeline::build(opts)` — full eval→convert→build→collect flow (from execute_builds_streaming)
- [ ] Implement `pipeline::bootstrap(opts)` — wraps bootstrap::bootstrap_fetch and bootstrap::resolve_packages
- [ ] Move FOD mismatch handling (`handle_fod_mismatch`, `auto_fix_hash`, `parse_fod_mismatch_error`) to crunch-pipeline
- [ ] Move `build_import_paths()` to crunch-pipeline
- [ ] Move `build_remote_pathinfo()` to crunch-store (called by pipeline)

## Phase 3: Gut main.rs

- [ ] Reduce main.rs to: parse args → match command → call pipeline function → format output
- [ ] Keep only CLI-specific code: clap Args, output formatting, exit codes
- [ ] Update `src/lib.rs` to re-export crunch-pipeline types
- [ ] Verify main.rs is under 200 lines

## Phase 4: Test migration

- [ ] Move pipeline integration tests from `tests/` to crunch-pipeline
- [ ] Verify assert_cmd integration tests still pass (binary interface unchanged)
- [ ] Add library-level tests in crunch-pipeline (no CLI, no subprocess)
