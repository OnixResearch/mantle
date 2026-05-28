## Phase 1: Implementation

- [x] [serial] Record current clean self-probe blocker evidence for stale rustup linker wrapper. Evidence: `evidence/current-blocker.md`. r[rust_package_planning.rust_topology_external_linker]
- [x] [serial] Add focused positive and negative tests for runtime `link-self-contained` argument handling. Evidence: `evidence/verification.md`, `rust_topology_runtime_args_disable_self_contained_linker_by_default`, `rust_topology_runtime_args_preserve_split_explicit_linker_mode`, `rust_topology_runtime_args_preserve_joined_explicit_linker_mode`. r[rust_package_planning.rust_topology_external_linker]
- [x] [serial] Disable rustc self-contained linker wrappers for topology execution unless selected unit args already set that mode. Evidence: `src/rust_plan.rs`, `rust_topology_runtime_args`. r[rust_package_planning.rust_topology_external_linker]
- [x] [serial] Run focused tests, self-probe, Cairn validation, and sync/archive readiness. Evidence: `evidence/verification.md`; pueue task `33` moved from missing rustup `ld-wrapper.sh` to Rust edition blocker in `nix-compat-derive`. r[rust_package_planning.rust_topology_external_linker]
