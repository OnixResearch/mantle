## ADDED Requirements

### Requirement: Native Rust unit graph construction

r[rust_package_planning.native_unit_graph] Mantle MUST construct supported Rust build unit graphs from native planning facts without invoking Cargo's unit graph oracle.

#### Scenario: supported units are produced natively

GIVEN native manifest, lockfile, source, and feature facts are ready
WHEN Mantle plans the Rust unit graph
THEN it MUST emit supported lib, bin, proc-macro, and custom-build units without calling Cargo.

#### Scenario: host and target edges are typed

GIVEN a target unit depends on build scripts, proc macros, and normal libraries
WHEN Mantle constructs unit graph edges
THEN host artifacts, build-script metadata, and target library artifacts MUST be represented as distinct typed edges.

#### Scenario: unsupported unit graph shapes fail closed

GIVEN dependency graph behavior falls outside Mantle's supported native unit subset
WHEN Mantle constructs the unit graph
THEN it MUST emit deterministic unsupported-unit-graph blockers before execution.
