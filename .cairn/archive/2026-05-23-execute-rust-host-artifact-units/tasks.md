# Tasks: execute-rust-host-artifact-units

## Implementation

- [x] [serial] r[rust_package_planning.unit_execution.host_artifacts.executes] Add host-artifact topology execution over explicit unit derivation graph nodes.
- [x] [serial] r[rust_package_planning.unit_execution.host_artifacts.binds] Bind host-produced artifacts into target consumed-host/dependency/`--extern` surfaces before target `rustc`.
- [x] [serial] r[rust_package_planning.unit_execution_receipts.host_artifacts.ordered] Emit ordered host+target execution receipts with stable receipt hash.
- [x] [serial] r[rust_package_planning.unit_execution_receipts.host_artifacts.cli.json] Expose host-artifact topology execution through `rust-plan` CLI JSON evidence.
- [x] [serial] r[rust_package_planning.unit_execution.host_artifacts.blockers] Add fail-closed tests for unsupported/missing host artifact material.

## Verification

- [x] [serial] r[rust_package_planning.unit_execution.host_artifacts.executes] Add a positive proc-macro host-artifact CLI fixture.
- [x] [serial] r[rust_package_planning.unit_execution.host_artifacts.blockers] Add a negative missing-host-producer or missing-host-artifact fixture.
- [x] [serial] r[rust_package_planning.unit_execution_receipts.host_artifacts.cli.json] Run focused Rust-plan and CLI tests plus Cairn validate/gates.
