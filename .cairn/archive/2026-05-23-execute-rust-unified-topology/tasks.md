# Tasks: execute-rust-unified-topology

## Implementation

- [x] [serial] r[rust_package_planning.unit_execution.topology.unified_cli] Add `rust-plan --execute-topology` as a mutually-exclusive execution rail requiring `--execution-output-root`.
- [x] [serial] r[rust_package_planning.unit_execution.topology.mixed_order] Execute supported host units first and supported target units in dependency order from one graph.
- [x] [serial] r[rust_package_planning.unit_execution.topology.binds_all_artifacts] Bind produced host artifacts, build-script metadata, and target library artifacts before dependent target rustc invocations.
- [x] [serial] r[rust_package_planning.unit_execution.topology.blockers] Fail closed for missing host producers, missing target dependency producers, graph blockers, and execution failures.

## Verification

- [x] [serial] r[rust_package_planning.unit_execution.topology.mixed_order] Add a positive CLI fixture with target lib/bin plus build-script host metadata through `--execute-topology`.
- [x] [serial] r[rust_package_planning.unit_execution.topology.blockers] Add a negative CLI fixture for a host-artifact graph shape that remains blocked on the target-only rail but succeeds through the unified rail.
- [x] [serial] r[rust_package_planning.unit_execution.topology.unified_cli] Run focused Rust-plan/CLI tests plus Cairn gates.
