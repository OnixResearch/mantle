# Rust Package Planning Specification Delta

## ADDED Requirements

### Requirement: Native build oracle target scope

r[rust_package_planning.native_build_oracle_target_scope] Mantle MUST scope normal build-mode Cargo oracle target-kind comparison to targets relevant to the normal build topology.

#### Scenario: Out-of-scope Cargo oracle targets do not block normal build planning

r[rust_package_planning.native_build_oracle_target_scope.normal_build]

- GIVEN Cargo metadata contains target kinds outside normal build execution, such as examples or benches
- WHEN Mantle plans normal build-mode Rust package/target facts
- THEN Mantle MUST NOT treat those out-of-scope target kinds as normal build-mode package blockers.
- AND Mantle MUST NOT claim support for executing those target kinds through normal build topology.
- AND existing unsupported dependency, source, cfg, and feature surfaces MUST remain fail-closed.
