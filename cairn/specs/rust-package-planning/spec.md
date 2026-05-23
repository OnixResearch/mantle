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

### Requirement: Rust unit execution from explicit derivation plans

r[rust_package_planning.unit_execution] Mantle MUST execute supported Rust package units from explicit `unit_derivation_graph` receipt nodes rather than invoking Cargo as a hidden build orchestrator.

#### Scenario: Supported unit executes from receipt material

r[rust_package_planning.unit_execution.supported_unit]

- GIVEN a `unit_derivation_graph` is ready and contains a supported `lib` or `bin` unit node
- WHEN Mantle executes that unit
- THEN Mantle MUST invoke `rustc` using the node's explicit arguments, environment, declared inputs, source-closure digest, dependency artifacts, host artifacts, and declared outputs.
- AND Mantle MUST NOT consult Cargo target directories, registry caches, git checkouts, or build orchestration as undeclared execution inputs.

#### Scenario: Bounded Cargo-free execution claim

r[rust_package_planning.unit_execution.bounded_claim]

- GIVEN Mantle successfully executes a supported explicit Rust unit
- WHEN Mantle reports the result
- THEN the report MAY claim Cargo-free execution for that explicit unit node only.
- AND the report MUST NOT claim full Cargo compatibility, doctest/test/example support, native-link probing correctness, rustc/compiler correctness, or full bootstrap correctness unless separate evidence exists.

### Requirement: Rust unit execution receipts

r[rust_package_planning.unit_execution_receipts] Mantle MUST emit deterministic receipts for Rust unit execution results.

#### Scenario: Output identity is receipt-bound

r[rust_package_planning.unit_execution_receipts.output_identity]

- GIVEN a Rust unit execution finishes successfully
- WHEN Mantle records the execution receipt
- THEN the receipt MUST bind the unit identity, source closure digest, dependency artifact digests, host artifact digests, toolchain identity, `rustc` argument digest, declared output paths, output artifact digests, execution status, and rebuild or reuse reason.
- AND the receipt MUST be deterministic across repeated executions with equivalent declared inputs.

#### Scenario: Failed execution preserves reviewable diagnostics

r[rust_package_planning.unit_execution_receipts.failure]

- GIVEN `rustc` execution fails for a supported unit
- WHEN Mantle records the failure
- THEN the receipt MUST include deterministic failure class, unit identity, input digest summary, and redacted diagnostics sufficient for review.
- AND the receipt MUST NOT include secrets, absolute temporary paths as hash material, or ambient environment dumps.

### Requirement: Rust unit execution fail-closed blockers

r[rust_package_planning.unit_execution_blockers] Mantle MUST reject Rust unit execution when required explicit material is absent or inconsistent.

#### Scenario: Missing execution material blocks before rustc

r[rust_package_planning.unit_execution_blockers.missing_material]

- GIVEN a supported Rust unit requires source closure material, dependency artifacts, host artifacts, declared output paths, or toolchain identity
- WHEN any required material is missing, unreadable, inconsistent with its digest, or not represented in the unit derivation receipt
- THEN Mantle MUST fail before invoking `rustc` with a deterministic blocker identifying the missing material class.
- AND Mantle MUST NOT search ambient Cargo caches or target directories to repair the missing material.

#### Scenario: Unsupported execution boundary remains explicit

r[rust_package_planning.unit_execution_blockers.unsupported_boundary]

- GIVEN a Rust unit requires execution behavior outside Mantle's supported subset
- WHEN Mantle evaluates the unit for execution
- THEN Mantle MUST emit a deterministic unsupported-boundary blocker rather than invoking Cargo or claiming a successful native execution.

### Requirement: Rust unit dependency-chain execution

r[rust_package_planning.unit_execution.dependency_chain] Mantle MUST execute a bounded Rust unit dependency chain from explicit `unit_derivation_graph` receipt nodes without invoking Cargo as a hidden build orchestrator.

#### Scenario: Consuming unit uses a Mantle-produced dependency artifact

r[rust_package_planning.unit_execution.dependency_chain.produced_artifact]

- GIVEN a ready `unit_derivation_graph` contains a supported producer `lib` unit and a supported consuming `lib` or `bin` unit whose dependency artifact references that producer package
- WHEN Mantle executes the bounded dependency chain
- THEN Mantle MUST execute the producer unit first using explicit receipt material.
- AND Mantle MUST rewrite the consumer's declared dependency artifact placeholder only to the artifact path produced by that producer execution.
- AND Mantle MUST invoke the consumer `rustc` with explicit args, env, source material, dependency artifact, and declared output material.
- AND Mantle MUST NOT consult Cargo target directories, registry caches, git checkouts, or build orchestration as undeclared dependency inputs.

### Requirement: Rust dependency-chain execution receipts

r[rust_package_planning.unit_execution_receipts.dependency_chain] Mantle MUST emit chain-level evidence that preserves ordered per-unit execution receipts.

#### Scenario: Chain receipt preserves producer and consumer identities

r[rust_package_planning.unit_execution_receipts.dependency_chain.ordered_units]

- GIVEN a bounded dependency chain executes successfully
- WHEN Mantle records the chain result
- THEN the chain evidence MUST include the ordered producer and consumer unit execution receipts.
- AND the consumer receipt MUST bind the dependency artifact digest for the producer artifact it consumed.
- AND the chain evidence MUST identify that the claim is limited to the explicit dependency edge, not full Cargo compatibility or a general Rust scheduler.

### Requirement: Rust dependency-chain execution blockers

r[rust_package_planning.unit_execution_blockers.dependency_chain] Mantle MUST reject unsupported or incomplete dependency-chain execution before invoking the consumer `rustc`.

#### Scenario: Missing or stale dependency material blocks the consumer

r[rust_package_planning.unit_execution_blockers.dependency_chain.missing_material]

- GIVEN a consuming unit requires a dependency artifact from a producer unit
- WHEN the producer unit is absent, unsupported, fails, emits no matching artifact, or the rewritten dependency artifact is missing/unreadable before the consumer is invoked
- THEN Mantle MUST emit a deterministic dependency-chain blocker.
- AND Mantle MUST NOT search ambient Cargo target directories or caches to repair the dependency material.

#### Scenario: Unsupported chain shape remains explicit

r[rust_package_planning.unit_execution_blockers.dependency_chain.unsupported_shape]

- GIVEN a dependency chain requires host artifacts, proc macros, build scripts, tests, doctests, examples, native-link probing, or more general scheduling than the bounded two-step rail supports
- WHEN Mantle evaluates the chain for execution
- THEN Mantle MUST emit a deterministic unsupported-chain blocker rather than invoking Cargo or claiming successful dependency-chain execution.
