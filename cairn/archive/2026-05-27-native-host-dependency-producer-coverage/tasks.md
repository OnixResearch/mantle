## Phase 1: Implementation

- [x] [serial] Record current self-probe blocker evidence for `rustversion@1.0.22` host dependency producer coverage. Evidence: `evidence/current-blocker.md`. r[rust_package_planning.native_host_dependency_producer_coverage]
- [x] [serial] Add focused positive coverage for a host unit whose selected dependency artifact is produced by a proc-macro host unit. Evidence: `evidence/verification.md`, `host_dependency_topology_accepts_proc_macro_host_producer`, `combined_unit_topology_orders_target_host_and_proc_macro_edges`. r[rust_package_planning.native_host_dependency_producer_coverage]
- [x] [serial] Add focused negative coverage for a host unit whose dependency artifact has neither target `lib` nor proc-macro host producer. Evidence: `evidence/verification.md`, `host_dependency_topology_blocks_missing_producer`, `combined_unit_topology_blocks_missing_target_host_artifact_producer`. r[rust_package_planning.native_host_dependency_producer_coverage]
- [x] [serial] Implement selected host dependency producer scheduling and binding in unified topology execution. Evidence: `src/rust_plan.rs`, `plan_host_dependency_topology`, `plan_combined_unit_topology_order`. r[rust_package_planning.native_host_dependency_producer_coverage]
- [x] [serial] Verify focused tests, self-probe blocker movement, Cairn validation, and sync/archive readiness. Evidence: `evidence/verification.md`; pueue task `20` self-probe moved from `missing-host-dependency-producer` to `rustc-failed`. r[rust_package_planning.native_host_dependency_producer_coverage]

## Phase 2: Review Remediation

- [x] [serial] Preserve unified topology execution coverage for selected host units that are not reachable from target roots. Evidence: `evidence/verification.md`, `combined_unit_topology_keeps_standalone_host_units`, pueue task `28` clean self-probe at `c3ccf629`. r[rust_package_planning.native_host_dependency_producer_coverage]
