## Phase 1: Implementation

- [x] [serial] Record current clean self-probe blocker evidence for missing `$RUSTC` in build-script execution. Evidence: `evidence/current-blocker.md`. r[rust_package_planning.native_build_script_execution_env]
- [x] [serial] Add focused positive and negative tests for build-script env/cwd planning. Evidence: `evidence/verification.md`, `build_script_child_env_sets_tool_target_and_manifest_dir`, `build_script_child_env_omits_manifest_dir_without_source_arg`. r[rust_package_planning.native_build_script_execution_env]
- [x] [serial] Bind deterministic build-script env vars and package-root cwd in `run_build_script_metadata`. Evidence: `src/rust_plan.rs`, `build_script_child_env`, `build_script_package_root`, `absolute_path_from`. r[rust_package_planning.native_build_script_execution_env]
- [x] [serial] Run focused tests, self-probe, Cairn validation, and sync/archive readiness. Evidence: `evidence/verification.md`; pueue task `39` moved from missing `$RUSTC` to proc-macro/dependency rustc failures after three successful build-script metadata runs. r[rust_package_planning.native_build_script_execution_env]
