# Tasks

## Spec

- [x] [serial] Add native unit graph construction requirement and design. r[rust_package_planning.native_unit_graph]

## Implementation

- [x] [serial] Construct target lib/bin units from native package and feature facts. r[rust_package_planning.native_unit_graph]
  - Evidence: `cargo test -p mantle --bin mantle rust_plan::tests::native_unit_graph -- --nocapture` (12 tests passed); `evidence/verification.md` records the durable self-probe summary.
- [x] [serial] Construct proc-macro and custom-build host units from native package and feature facts. r[rust_package_planning.native_unit_graph]
  - Evidence: `cargo test -p mantle --bin mantle rust_plan::tests::native_host -- --nocapture` (16 tests passed); `evidence/verification.md` records `host_ready=true`, `metadata_runs=59`, and `topology_execution_status=success` from the durable self-probe summary.
- [x] [serial] Build typed dependency, host artifact, and build-script metadata edges. r[rust_package_planning.native_unit_graph]
  - Evidence: `cargo test -p mantle --bin mantle rust_plan::tests::native_host -- --nocapture` (16 tests passed); `evidence/verification.md` records the durable self-probe summary.
- [x] [serial] Define stable native unit IDs independent of Cargo unit indices. r[rust_package_planning.native_unit_graph]
  - Evidence: `native_unit_graph_uses_stable_native_unit_identity` passed in the focused native graph run.
- [x] [serial] Emit fail-closed blockers for unsupported graph shapes. r[rust_package_planning.native_unit_graph]
  - Evidence: `native_unit_graph_blocks_*` and `native_target_cfg_predicate_scope_*` tests passed in focused runs.

## Verification

- [x] [serial] Run native graph unit tests, Cargo oracle parity tests, topology execution smoke tests, and Cairn validation. r[rust_package_planning.native_unit_graph]
  - Evidence: `evidence/verification.md` records the focused test outcomes, `cairn validate --root .` (`valid: true`), and the self-probe summary (`topology_execution_status=success`, `executions=580`). Additional topology smoke tests passed: `rust_plan_cli_executes_vendored_registry_dependency_in_unified_topology` and `rust_plan_cli_executes_vendored_registry_proc_macro_in_unified_topology`.
