# Tasks: execute-rust-build-script-metadata

## Implementation

- [x] [serial] r[rust_package_planning.unit_execution.build_script_metadata.executes] Execute compiled custom-build host artifacts with deterministic OUT_DIR.
- [x] [serial] r[rust_package_planning.unit_execution.build_script_metadata.captures] Capture supported build-script metadata lines into deterministic receipts.
- [x] [serial] r[rust_package_planning.unit_execution.build_script_metadata.binds] Bind OUT_DIR, rustc-env, and rustc-cfg metadata into target rustc execution.
- [x] [serial] r[rust_package_planning.unit_execution.build_script_metadata.blockers] Fail closed on missing or malformed build-script metadata.
- [x] [serial] r[rust_package_planning.unit_execution_receipts.build_script_metadata.cli] Expose build-script metadata evidence through host-artifact topology CLI JSON receipts.

## Verification

- [x] [serial] r[rust_package_planning.unit_execution.build_script_metadata.executes] Add a positive build.rs CLI fixture that writes OUT_DIR material and emits cfg/env metadata.
- [x] [serial] r[rust_package_planning.unit_execution.build_script_metadata.blockers] Add a negative malformed build-script metadata fixture.
- [x] [serial] r[rust_package_planning.unit_execution_receipts.build_script_metadata.cli] Run focused Rust-plan and CLI tests plus Cairn validate/gates.
