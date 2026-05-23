### Requirement: Unified Rust topology execution

r[rust_package_planning.unit_execution.topology] Mantle MUST expose a bounded Cargo-free Rust topology execution rail for supported mixed host and target unit graphs.

#### Scenario: Unified topology CLI executes mixed units

r[rust_package_planning.unit_execution.topology.unified_cli]

- GIVEN a Rust package graph contains supported target units and optional supported host units
- WHEN `rust-plan --execute-topology` is requested with an execution output root
- THEN Mantle MUST emit a combined receipt containing the retained Rust plan and unified topology execution evidence.

#### Scenario: Host and target units execute in explicit order

r[rust_package_planning.unit_execution.topology.mixed_order]

- GIVEN supported host units and supported target units appear in the derivation graph
- WHEN Mantle executes the unified topology rail
- THEN Mantle MUST execute supported host units before target consumers and MUST execute supported target units in dependency order.

#### Scenario: All produced artifact classes are bound

r[rust_package_planning.unit_execution.topology.binds_all_artifacts]

- GIVEN target units consume proc-macro/build-script host artifacts, build-script metadata, or target library artifacts
- WHEN Mantle invokes dependent target rustc units through the unified topology rail
- THEN Mantle MUST bind the produced artifact paths and captured metadata into deterministic rustc inputs before invocation.

#### Scenario: Unsupported or incomplete topology fails closed

r[rust_package_planning.unit_execution.topology.blockers]

- GIVEN the graph is not ready, a required producer is missing, or a unit execution fails
- WHEN Mantle executes the unified topology rail
- THEN Mantle MUST return a structured blocker with ordered partial execution evidence and MUST NOT continue to dependent rustc invocations.
