# Rust Package Planning Specification Delta

## ADDED Requirements

### Requirement: Native dependency feature edge scope

r[rust_package_planning.native_dependency_feature_edge_scope] Mantle MUST scope native package dependency planning to dependency feature/default-feature selections represented by the captured build unit graph for the current invocation.

#### Scenario: Disabled default dependency features do not require optional sources

GIVEN a native registry dependency package has a default feature that selects an optional dependency
WHEN the current build unit graph selects that package with `default-features = false`
THEN Mantle MUST NOT select the package's default optional dependency in native package dependency facts
AND MUST NOT require a path or declared registry source for that unselected optional dependency
AND MUST NOT add that unselected optional dependency to the normal build topology dependency graph.

#### Scenario: Selected default dependency features still bind sources

GIVEN a native registry dependency package has a default feature that selects an optional dependency
WHEN the current build unit graph selects that package's default feature
THEN Mantle MUST keep the selected optional dependency in native package dependency facts
AND MUST require the selected dependency to resolve through the bounded path or declared vendored-registry source fragment before rustc execution.

#### Scenario: Bounded build dependency feature metadata does not block source edges

GIVEN a build dependency edge uses `features`, `default-features`, or `optional` metadata
WHEN the dependency source resolves through a path dependency or declared vendored-registry source and the selected package feature facts come from the captured build unit graph
THEN Mantle MUST NOT emit `unsupported-build-dependency-options` solely for that feature metadata
AND MUST still fail closed for unsupported source kinds or unresolved selected optional dependencies.
