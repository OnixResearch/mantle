### Requirement: Rust build-script native link metadata binding

r[rust_package_planning.unit_execution.build_script_metadata.link_binding] Mantle MUST bind bounded native link metadata emitted by explicit custom-build host units into downstream Cargo-free rustc target execution.

#### Scenario: Build-script link-search metadata is bound

r[rust_package_planning.unit_execution.build_script_metadata.link_search_binding]

- GIVEN a target unit consumes a custom-build host unit that emits supported `cargo:rustc-link-search` metadata
- WHEN Mantle executes the host-artifact topology rail
- THEN Mantle MUST append deterministic `-L` rustc arguments before invoking the target rustc.

#### Scenario: Build-script link-lib metadata is bound

r[rust_package_planning.unit_execution.build_script_metadata.link_lib_binding]

- GIVEN a target unit consumes a custom-build host unit that emits supported `cargo:rustc-link-lib` metadata
- WHEN Mantle executes the host-artifact topology rail
- THEN Mantle MUST append deterministic `-l` rustc arguments before invoking the target rustc.

#### Scenario: Unsupported link metadata blocks fail closed

r[rust_package_planning.unit_execution.build_script_metadata.link_metadata_blockers]

- GIVEN a custom-build host unit emits malformed, ambiguous, or unsupported native link metadata
- WHEN Mantle parses the build-script metadata
- THEN Mantle MUST return a structured blocker before invoking target rustc.
