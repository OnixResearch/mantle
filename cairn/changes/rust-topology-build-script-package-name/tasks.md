## Phase 1: Implementation

- [ ] [serial] Record current review finding and baseline focused test evidence. Evidence: `evidence/current-blocker.md`. r[rust_package_planning.native_build_script_package_name_env]
- [ ] [serial] Add focused positive and fallback tests for build-script `CARGO_PKG_NAME`. Evidence: `evidence/verification.md`, focused rust-plan tests. r[rust_package_planning.native_build_script_package_name_env]
- [ ] [serial] Carry package-derived name into native derivation env and use it in `build_script_child_env`. Evidence: `src/rust_plan.rs`. r[rust_package_planning.native_build_script_package_name_env]
- [ ] [serial] Run focused tests, self-probe, Cairn validation, and sync/archive readiness. Evidence: `evidence/verification.md`. r[rust_package_planning.native_build_script_package_name_env]
