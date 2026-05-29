## ADDED Requirements

### Requirement: Native Rust feature resolution

r[rust_package_planning.native_feature_resolution] Mantle MUST compute selected Rust package features and optional dependency activation without invoking Cargo for the supported planning subset.

#### Scenario: default and explicit features resolve deterministically

GIVEN a workspace package with default features, explicit CLI features, and transitive feature edges
WHEN Mantle runs native feature resolution
THEN it MUST emit deterministic selected feature sets for each planned package and unit.

#### Scenario: optional dependencies activate through features

GIVEN a package feature enables an optional dependency
WHEN that feature is selected
THEN Mantle MUST include the dependency in the resolved package graph
AND absent feature selection MUST leave the optional dependency unselected.

#### Scenario: unsupported feature surface fails closed

GIVEN feature resolution encounters unsupported target cfg, resolver, or source replacement behavior
WHEN Mantle plans native units
THEN it MUST emit a deterministic unsupported-feature-resolution blocker before unit execution.
