## ADDED Requirements

### Requirement: Native selected host-unit planning

r[rust_package_planning.native_selected_host_units] Mantle MUST plan native host units from Cargo-selected host-unit facts instead of every manifest-visible host target.

#### Scenario: Selected proc-macro host unit remains planned

GIVEN Cargo's unit graph selects a native proc-macro host unit
WHEN Mantle plans native host units
THEN Mantle MUST include that proc-macro host unit in the native host-unit graph.
AND Mantle MUST preserve selected normal dependency artifacts for that host unit.

#### Scenario: Unselected proc-macro host unit is not planned

GIVEN a package metadata record contains a proc-macro target that is absent from Cargo's selected unit graph
WHEN Mantle plans native host units
THEN Mantle MUST NOT include that proc-macro target as a native host unit.
AND Mantle MUST NOT execute it or report blockers from its unselected dependencies.

#### Scenario: Same-package selected build script still feeds proc macro

GIVEN Cargo selects both a package's custom-build host unit and its proc-macro host unit
WHEN Mantle plans native host units
THEN Mantle MUST keep the custom-build host unit ordered before the same-package proc-macro host unit.
AND Mantle MUST continue to expose the build-script metadata and `OUT_DIR` to that selected proc-macro host unit.

#### Scenario: Jiff-static false frontier moves

GIVEN topology execution currently reaches an unselected `jiff-static@0.2.23` host proc-macro and fails on unresolved `quote` and `syn`
WHEN selected host-unit planning is applied
THEN self-probe verification MUST show that this unselected `jiff-static` blocker no longer stops the topology.
AND any remaining blocker MUST be recorded with deterministic class and evidence.
