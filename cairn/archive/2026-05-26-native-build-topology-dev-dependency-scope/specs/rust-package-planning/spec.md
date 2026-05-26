# Rust Package Planning Specification Delta

## ADDED Requirements

### Requirement: Native build topology dev-dependency scope

r[rust_package_planning.native_build_topology_dev_dependency_scope] Mantle MUST keep normal build-mode Rust package/target and unit topology planning separate from dev-dependency-only option surfaces.

#### Scenario: Dev-dependency options do not block normal build topology

r[rust_package_planning.native_build_topology_dev_dependency_scope.normal_build]

- GIVEN a package has dev-dependencies with feature/default-feature/optional/target/workspace options
- WHEN Mantle plans or executes the normal build topology
- THEN Mantle MUST NOT treat those dev-only options as normal build-mode package blockers.
- AND Mantle MUST NOT execute dev-dependency units through the normal build topology.
- AND Mantle MUST NOT claim `native_rust_dev_dependency_test_topology_execution` from the normal build topology.

#### Scenario: Dev-test execution remains explicit

r[rust_package_planning.native_build_topology_dev_dependency_scope.dev_test_explicit]

- GIVEN dev-dependency source material is present
- WHEN Mantle needs to execute dev/test units
- THEN Mantle MUST use an explicit dev-dependency test topology rail rather than silently widening normal build topology execution.
