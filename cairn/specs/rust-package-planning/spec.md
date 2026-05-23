# Rust Package Planning Specification

## Purpose

Defines the `rust-package-planning` capability.

## Requirements

### Requirement: Cargo-oracle parity for Mantle Rust planning

r[rust_package_planning.cargo_oracle_parity] Mantle MUST introduce Rust package planning through an oracle-compatible phase that records Cargo's package metadata and unit graph before replacing Cargo's planner.

#### Scenario: Cargo unit graph is captured as reviewable evidence

r[rust_package_planning.cargo_oracle_parity.capture]

- GIVEN a Rust workspace is selected for Mantle Rust package planning
- WHEN the oracle phase is used
- THEN Mantle MUST capture the relevant `cargo metadata` and `cargo build --unit-graph` material into a normalized, reviewable plan or receipt.
- AND the receipt MUST identify the workspace root, lockfile identity, Cargo/rustc toolchain identity, target triples, selected profiles, enabled features, and unit graph digest.

#### Scenario: Mantle-computed graph fragments compare against Cargo

r[rust_package_planning.cargo_oracle_parity.compare]

- GIVEN Mantle implements native Rust planning for a supported subset
- WHEN Cargo oracle material is available for the same workspace, target, profile, and feature set
- THEN Mantle MUST compare its computed package, feature, target, and unit graph fragments against Cargo's oracle output.
- AND mismatches MUST fail closed with deterministic diagnostics before any replacement build claim is made.

### Requirement: Explicit Rust source closure ownership

r[rust_package_planning.source_closure] Mantle MUST represent Rust package sources as explicit source-closure inputs before those sources become build units.

#### Scenario: Registry, git, and path dependencies are identified

r[rust_package_planning.source_closure.identities]

- GIVEN a Cargo lockfile and manifests describe registry, git, and path dependencies
- WHEN Mantle constructs a Rust source closure
- THEN each source input MUST carry its package identity, source kind, lockfile identity when applicable, resolved revision or checksum material when applicable, and content-addressed source digest.

#### Scenario: Ambient Cargo caches are not trusted build inputs

r[rust_package_planning.source_closure.offline]

- GIVEN a Rust package build runs through Mantle
- WHEN source material is needed
- THEN Mantle MUST consume declared source inputs from the source closure rather than relying on ambient Cargo registry, git, or target-directory caches.
- AND missing source-closure material MUST produce a deterministic planning/build blocker.

### Requirement: Mantle Rust unit derivation graph

r[rust_package_planning.unit_derivation_graph] Mantle MUST lower supported Rust package units into explicit Mantle derivations rather than invoking Cargo as the hidden build orchestrator.

#### Scenario: Supported Rust units become derivation nodes

r[rust_package_planning.unit_derivation_graph.units]

- GIVEN a supported Rust package unit is planned
- WHEN Mantle emits the build graph
- THEN the unit MUST become an explicit derivation node with declared inputs, environment, output paths, target/profile identity, feature cfgs, dependency artifacts, and reviewable `rustc` arguments.

#### Scenario: Per-unit outputs explain rebuilds and cache identity

r[rust_package_planning.unit_derivation_graph.cache_identity]

- GIVEN a Rust unit derivation is built or reused
- WHEN Mantle records the build result
- THEN the receipt MUST bind the unit identity, source closure digest, dependency artifact digests, toolchain identity, `rustc` argument digest, and output artifact digest.
- AND the receipt MUST be sufficient to explain why the unit was rebuilt or reused.

### Requirement: Host and target unit separation

r[rust_package_planning.host_target_split] Mantle MUST model host and target Rust units separately, including build scripts and proc macros.

#### Scenario: Build scripts are host units with explicit outputs

r[rust_package_planning.host_target_split.build_scripts]

- GIVEN a package contains a `build.rs`
- WHEN Mantle supports that package
- THEN the build script MUST be compiled and executed as a host unit.
- AND its generated outputs, `OUT_DIR`, `cargo:rustc-cfg`, `cargo:rustc-env`, `cargo:rustc-link-lib`, `cargo:rustc-link-search`, and rerun metadata MUST be captured as explicit receipt material before target units consume them.

#### Scenario: Proc macros are host artifacts consumed by target units

r[rust_package_planning.host_target_split.proc_macros]

- GIVEN a dependency target is a proc macro
- WHEN Mantle builds a target unit that depends on it
- THEN the proc macro MUST be planned and built for the host execution environment while the consuming unit remains tied to the target environment.
- AND host/target confusion MUST fail closed.

### Requirement: Fail-closed Cargo replacement boundaries

r[rust_package_planning.fail_closed_boundaries] Mantle MUST reject or mark unsupported Cargo behavior explicitly instead of silently falling back to opaque Cargo builds.

#### Scenario: Unsupported Cargo behavior produces blockers

r[rust_package_planning.fail_closed_boundaries.unsupported]

- GIVEN a workspace uses Cargo behavior outside Mantle's currently supported Rust planning subset
- WHEN Mantle plans or builds the workspace
- THEN Mantle MUST emit a deterministic unsupported-boundary diagnostic identifying the unsupported behavior class.
- AND Mantle MUST NOT claim Cargo-free planning or Cargo-free build success for that workspace.

#### Scenario: Rust planner claims remain bounded

r[rust_package_planning.fail_closed_boundaries.non_claims]

- GIVEN a Rust package build succeeds through Mantle's Rust planner
- WHEN Mantle reports the result
- THEN the report MUST NOT claim rustc correctness, Cargo ecosystem completeness, semantic equivalence for unsupported target kinds, native-link correctness beyond declared metadata, or full bootstrap correctness unless separate evidence exists.
