### Requirement: Rust build-script metadata execution

r[rust_package_planning.unit_execution.build_script_metadata] Mantle MUST execute bounded Rust build-script metadata from explicit custom-build host units without invoking Cargo as the build orchestrator.

#### Scenario: Custom-build host unit emits deterministic metadata

r[rust_package_planning.unit_execution.build_script_metadata.executes]

- GIVEN `unit_derivation_graph` is ready and contains a supported `custom-build` host unit plus a supported target consumer
- WHEN build-script metadata execution is requested through the host-artifact topology rail
- THEN Mantle MUST compile the custom-build host unit and run the produced executable with a deterministic `OUT_DIR`.
- AND Mantle MUST NOT invoke Cargo to orchestrate the build-script execution.

#### Scenario: Build-script metadata is captured

r[rust_package_planning.unit_execution.build_script_metadata.captures]

- GIVEN a custom-build host executable emits supported `cargo:` metadata lines
- WHEN Mantle runs that executable
- THEN Mantle MUST capture `rustc-cfg`, `rustc-env`, `rustc-link-lib`, `rustc-link-search`, and `rerun-if-changed` surfaces into deterministic receipt metadata.
- AND Mantle MUST hash the captured metadata using BLAKE3 evidence.

#### Scenario: Build-script metadata is bound into target rustc

r[rust_package_planning.unit_execution.build_script_metadata.binds]

- GIVEN a target unit consumes a build-script host artifact
- WHEN Mantle executes that target unit after the custom-build host run
- THEN Mantle MUST bind `OUT_DIR`, `rustc-env`, and `rustc-cfg` metadata into the target rustc environment or arguments before invoking rustc.

#### Scenario: Build-script metadata blocks fail closed

r[rust_package_planning.unit_execution.build_script_metadata.blockers]

- GIVEN build-script metadata is required by a target consumer
- WHEN the custom-build executable is missing, fails, emits malformed metadata, or its generated metadata is not available
- THEN Mantle MUST return a structured blocker before invoking target rustc.

### Requirement: Build-script metadata execution receipts

r[rust_package_planning.unit_execution_receipts.build_script_metadata] Mantle MUST emit deterministic JSON evidence for build-script metadata execution.

#### Scenario: CLI JSON includes build-script metadata evidence

r[rust_package_planning.unit_execution_receipts.build_script_metadata.cli]

- GIVEN build-script metadata execution is requested through `rust-plan --execute-host-artifact-topology`
- WHEN Mantle emits the combined JSON receipt
- THEN the receipt MUST include the retained rust plan, ordered host and target unit executions, build-script run metadata, blockers when present, and stable BLAKE3 receipt hashes.
