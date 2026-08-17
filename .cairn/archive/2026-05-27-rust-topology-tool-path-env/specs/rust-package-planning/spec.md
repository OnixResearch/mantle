# Rust Package Planning Specification Delta

## ADDED Requirements

### Requirement: Rust topology tool PATH environment

r[rust_package_planning.rust_topology_tool_path_env] Mantle MUST provide a bounded tool PATH to Rust topology child processes when the caller supplies one.

#### Scenario: Rustc child keeps invocation tool PATH

GIVEN the caller invokes Rust topology execution with a non-empty `PATH`
WHEN Mantle invokes a selected `rustc` unit after clearing ambient environment
THEN Mantle MUST pass that `PATH` to the `rustc` child process.
AND Mantle MUST still pass only explicit derivation environment keys plus the tool PATH.

#### Scenario: Build-script metadata child keeps invocation tool PATH

GIVEN a selected custom-build unit is rebuilt and then executed for metadata
WHEN Mantle invokes the produced build-script executable after clearing ambient environment
THEN Mantle MUST pass the same non-empty tool `PATH` to the build-script child process.
AND Mantle MUST still set required metadata variables such as `OUT_DIR` explicitly.

#### Scenario: Empty tool PATH stays absent

GIVEN the caller has no usable `PATH`
WHEN Mantle constructs a Rust topology child environment
THEN Mantle MUST omit `PATH` instead of fabricating host-specific defaults.
AND downstream tool failures MUST remain deterministic execution blockers.
