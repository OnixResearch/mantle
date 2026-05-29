# Tasks

## Spec

- [x] [serial] Add native unit graph construction requirement and design. r[rust_package_planning.native_unit_graph]

## Implementation

- [x] [serial] Construct target lib/bin units from native package and feature facts. r[rust_package_planning.native_unit_graph]
  - Evidence: `cargo test -p mantle --bin mantle rust_plan::tests::native_unit_graph -- --nocapture` (12 tests passed) and self-probe task 493 reports `unit_ready=true`, `native_units=477`.
- [ ] [serial] Construct proc-macro and custom-build host units from native package and feature facts. r[rust_package_planning.native_unit_graph]
- [x] [serial] Build typed dependency, host artifact, and build-script metadata edges. r[rust_package_planning.native_unit_graph]
  - Evidence: `cargo test -p mantle --bin mantle rust_plan::tests::native_host -- --nocapture` (15 tests passed) and self-probe task 493 reports `host_ready=true`, `metadata_runs=50`.
- [x] [serial] Define stable native unit IDs independent of Cargo unit indices. r[rust_package_planning.native_unit_graph]
  - Evidence: `native_unit_graph_uses_stable_native_unit_identity` passed in the focused native graph run.
- [x] [serial] Emit fail-closed blockers for unsupported graph shapes. r[rust_package_planning.native_unit_graph]
  - Evidence: `native_unit_graph_blocks_*` and `native_target_cfg_predicate_scope_*` tests passed in focused runs.

## Verification

- [ ] [serial] Run native graph unit tests, Cargo oracle parity tests, topology execution smoke tests, and Cairn validation. r[rust_package_planning.native_unit_graph]
