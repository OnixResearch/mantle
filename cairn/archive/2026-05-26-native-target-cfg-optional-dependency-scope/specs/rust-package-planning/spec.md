# Rust Package Planning Specification Delta

## ADDED Requirements

### Requirement: Native target-cfg optional dependency scope

r[rust_package_planning.native_target_cfg_optional_dependency_scope] Mantle MUST distinguish selected target cfg tables from selected optional dependencies when planning native build topology dependency facts.

#### Scenario: Unselected optional cfg dependencies do not require sources

GIVEN a native package manifest with a supported target cfg dependency table
WHEN the cfg table is selected for the active target but an optional dependency in that table is not selected by native selected feature facts
THEN Mantle MUST record the dependency decision as `not-selected-optional`
AND MUST NOT require a path or declared registry source for that optional dependency
AND MUST NOT add that dependency to the native path dependency graph.

#### Scenario: Required cfg dependencies still fail closed

GIVEN a native package manifest with a supported target cfg dependency table
WHEN a non-optional dependency in that selected table cannot be resolved through a path dependency or declared registry source
THEN Mantle MUST emit an `unsupported-target-cfg-dependency` blocker before rustc execution.
