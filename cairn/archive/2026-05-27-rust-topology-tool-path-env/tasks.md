## Phase 1: Implementation

- [x] [serial] Record current clean self-probe blocker evidence for rustc wrapper PATH failure. Evidence: `evidence/current-blocker.md`. r[rust_package_planning.rust_topology_tool_path_env]
- [x] [serial] Add focused positive and negative tests for Rust topology child environment PATH construction. Evidence: `evidence/verification.md`, `rust_topology_child_env_preserves_non_empty_inherited_path`, `rust_topology_child_env_omits_empty_inherited_path`, `rust_topology_child_env_prefers_explicit_derivation_path`. r[rust_package_planning.rust_topology_tool_path_env]
- [x] [serial] Preserve bounded tool PATH for rustc and build-script metadata child processes after `env_clear()`. Evidence: `src/rust_plan.rs`, `rust_topology_child_env`, `apply_rust_topology_child_env`. r[rust_package_planning.rust_topology_tool_path_env]
- [x] [serial] Run focused tests, self-probe, Cairn validation, and sync/archive readiness. Evidence: `evidence/verification.md`; pueue task `29` moved from `env: 'bash': No such file or directory` to missing rustup `ld-wrapper.sh`. r[rust_package_planning.rust_topology_tool_path_env]
