## Phase 1: Implementation

- [ ] [serial] Record current clean self-probe blocker evidence for missing `$RUSTC` in build-script execution. Evidence: `evidence/current-blocker.md`. r[rust_package_planning.native_build_script_execution_env]
- [ ] [serial] Add focused positive and negative tests for build-script env/cwd planning. Evidence: `evidence/verification.md`, focused rust-plan tests. r[rust_package_planning.native_build_script_execution_env]
- [ ] [serial] Bind deterministic build-script env vars and package-root cwd in `run_build_script_metadata`. Evidence: `src/rust_plan.rs`. r[rust_package_planning.native_build_script_execution_env]
- [ ] [serial] Run focused tests, self-probe, Cairn validation, and sync/archive readiness. Evidence: `evidence/verification.md`. r[rust_package_planning.native_build_script_execution_env]
