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

### Requirement: Rust dependency-chain CLI evidence

r[rust_package_planning.unit_execution.dependency_chain.cli] Mantle MUST expose bounded Rust dependency-chain execution as reviewable `rust-plan` CLI evidence.

#### Scenario: CLI executes a bounded explicit dependency edge

r[rust_package_planning.unit_execution.dependency_chain.cli.executes]

- GIVEN `mantle rust-plan` captures a ready `unit_derivation_graph` with a supported producer `lib` unit and a supported consuming unit
- WHEN the dependency-chain execution CLI flag is requested with an explicit execution output root
- THEN Mantle MUST execute the bounded dependency chain through the explicit graph executor.
- AND Mantle MUST NOT invoke Cargo as the build orchestrator for producer or consumer execution.

### Requirement: Rust dependency-chain CLI receipts

r[rust_package_planning.unit_execution_receipts.dependency_chain.cli] Mantle MUST emit CLI receipts that preserve the captured plan receipt and the dependency-chain execution receipt.

#### Scenario: CLI receipt preserves ordered chain evidence

r[rust_package_planning.unit_execution_receipts.dependency_chain.cli.ordered_units]

- GIVEN dependency-chain execution is requested through the `rust-plan` CLI
- WHEN Mantle reports the result
- THEN the JSON receipt MUST include the captured Rust plan and the chain-level receipt with ordered producer and consumer unit execution receipts.
- AND the receipt MUST preserve output artifact BLAKE3 digests and the bounded explicit dependency-edge claim.

### Requirement: Rust target-unit topology execution

r[rust_package_planning.unit_execution.target_topology] Mantle MUST execute a bounded target-only Rust unit topology from explicit unit derivation graph evidence without invoking Cargo as the build orchestrator.

#### Scenario: Executes supported target units in dependency order

r[rust_package_planning.unit_execution.target_topology.executes]

- GIVEN `unit_derivation_graph` is ready and contains supported target `lib`/`bin` units whose dependency artifacts are produced by supported target `lib` units in the same graph
- WHEN target topology execution is requested
- THEN Mantle MUST execute producer units before consumers using the explicit derivation args/env.
- AND Mantle MUST rebind produced `.rlib` artifacts into downstream dependency artifact, input, and `--extern` surfaces before invoking each consumer `rustc`.
- AND Mantle MUST NOT invoke Cargo as the build orchestrator for the topology execution.

### Requirement: Rust target-unit topology blockers

r[rust_package_planning.unit_execution.target_topology.blockers] Mantle MUST fail closed with deterministic blockers for unsupported or incomplete topology shapes before invoking an affected consumer `rustc`.

#### Scenario: Blocks unsupported target topology shapes

r[rust_package_planning.unit_execution.target_topology.blockers.unsupported]

- GIVEN the graph is not ready, requires host/proc-macro/build-script artifacts, contains missing producer libs, cycles, missing produced artifacts, or unsupported target modes/kinds
- WHEN target topology execution is requested
- THEN Mantle MUST emit a deterministic topology blocker receipt.
- AND Mantle MUST NOT claim successful target-topology execution for the unsupported shape.

### Requirement: Rust target-unit topology receipts

r[rust_package_planning.unit_execution_receipts.target_topology] Mantle MUST emit a target topology execution receipt that preserves ordered per-unit execution evidence and deterministic blocker evidence.

#### Scenario: Receipt preserves ordered topology evidence

r[rust_package_planning.unit_execution_receipts.target_topology.ordered]

- GIVEN target topology execution is requested
- WHEN Mantle reports the result
- THEN the receipt MUST include ordered unit execution receipts, output artifact BLAKE3 digests, dependency artifact BLAKE3 digests for consumers, a bounded target-only claim, an optional blocker, and a stable receipt hash.

### Requirement: Rust target-unit topology CLI receipts

r[rust_package_planning.unit_execution_receipts.target_topology.cli] Mantle MUST expose target topology execution as reviewable `rust-plan` CLI JSON evidence.

#### Scenario: CLI emits captured plan plus topology execution receipt

r[rust_package_planning.unit_execution_receipts.target_topology.cli.json]

- GIVEN `mantle rust-plan` captures a ready target-only unit graph
- WHEN the target topology CLI flag is requested with an explicit execution output root
- THEN Mantle MUST emit JSON containing the captured Rust plan receipt and the target topology execution receipt.

### Requirement: Rust host-artifact topology execution

r[rust_package_planning.unit_execution.host_artifacts] Mantle MUST execute a bounded Rust host-artifact topology from explicit unit derivation graph evidence without invoking Cargo as the build orchestrator.

#### Scenario: Host artifacts execute before target consumers

r[rust_package_planning.unit_execution.host_artifacts.executes]

- GIVEN `unit_derivation_graph` is ready and contains supported host `proc-macro` or `custom-build` units plus supported target `lib`/`bin` consumers
- WHEN host-artifact topology execution is requested
- THEN Mantle MUST execute supported host units before target consumers using explicit derivation args/env.
- AND Mantle MUST NOT invoke Cargo as the build orchestrator for host or target execution.

#### Scenario: Host artifacts bind into target execution material

r[rust_package_planning.unit_execution.host_artifacts.binds]

- GIVEN a target unit consumes a host artifact represented in `consumed_host_artifacts` and matching dependency surfaces
- WHEN the producing host unit succeeds
- THEN Mantle MUST bind the host-produced artifact path into target consumed-host artifacts, derivation inputs, and matching `--extern` dependency surfaces before invoking target `rustc`.
- AND the target receipt MUST bind the host artifact digest for the produced artifact it consumed.

### Requirement: Rust host-artifact execution blockers

r[rust_package_planning.unit_execution.host_artifacts.blockers] Mantle MUST fail closed with deterministic blockers for unsupported or incomplete host-artifact execution before invoking an affected target `rustc`.

#### Scenario: Missing or unsupported host material blocks target execution

r[rust_package_planning.unit_execution.host_artifacts.blockers.missing_material]

- GIVEN a target unit consumes a host artifact
- WHEN the host producer is absent, unsupported, fails, emits no matching artifact, or the rebound host artifact is missing/unreadable
- THEN Mantle MUST emit a deterministic host-artifact topology blocker.
- AND Mantle MUST NOT search ambient Cargo target directories or caches to repair host material.

### Requirement: Rust host-artifact execution receipts

r[rust_package_planning.unit_execution_receipts.host_artifacts] Mantle MUST emit host-artifact topology execution receipts that preserve ordered host and target per-unit execution evidence.

#### Scenario: Receipt preserves ordered host and target evidence

r[rust_package_planning.unit_execution_receipts.host_artifacts.ordered]

- GIVEN host-artifact topology execution is requested
- WHEN Mantle reports the result
- THEN the receipt MUST include ordered host and target unit execution receipts, output artifact BLAKE3 digests, host artifact BLAKE3 digests for consumers, a bounded host-artifact claim, an optional blocker, and a stable receipt hash.

### Requirement: Rust host-artifact CLI receipts

r[rust_package_planning.unit_execution_receipts.host_artifacts.cli] Mantle MUST expose host-artifact topology execution as reviewable `rust-plan` CLI JSON evidence.

#### Scenario: CLI emits captured plan plus host-artifact execution receipt

r[rust_package_planning.unit_execution_receipts.host_artifacts.cli.json]

- GIVEN `mantle rust-plan` captures a ready unit graph with supported host artifacts and target consumers
- WHEN the host-artifact topology CLI flag is requested with an explicit execution output root
- THEN Mantle MUST emit JSON containing the captured Rust plan receipt and the host-artifact topology execution receipt.

### Requirement: Rust build-script metadata execution

r[rust_package_planning.unit_execution.build_script_metadata] Mantle MUST execute bounded Rust build-script metadata from explicit custom-build host units without invoking Cargo as the build orchestrator.

#### Scenario: Custom-build host unit emits deterministic metadata

r[rust_package_planning.unit_execution.build_script_metadata.executes]

- GIVEN `unit_derivation_graph` is ready and contains a supported `custom-build` host unit plus a supported target consumer
- WHEN build-script metadata execution is requested through the host-artifact topology rail
- THEN Mantle MUST compile the custom-build host unit and run the produced executable with a deterministic `OUT_DIR`.
- AND Mantle MUST NOT invoke Cargo to orchestrate the build-script execution.

#### Scenario: Build-script metadata is captured

r[rust_package_planning.unit_execution.build_script_metadata.captures]

- GIVEN a custom-build host executable emits supported `cargo:` metadata lines
- WHEN Mantle runs that executable
- THEN Mantle MUST capture `rustc-cfg`, `rustc-env`, `rustc-link-lib`, `rustc-link-search`, and `rerun-if-changed` surfaces into deterministic receipt metadata.
- AND Mantle MUST hash the captured metadata using BLAKE3 evidence.

#### Scenario: Build-script metadata is bound into target rustc

r[rust_package_planning.unit_execution.build_script_metadata.binds]

- GIVEN a target unit consumes a build-script host artifact
- WHEN Mantle executes that target unit after the custom-build host run
- THEN Mantle MUST bind `OUT_DIR`, `rustc-env`, and `rustc-cfg` metadata into the target rustc environment or arguments before invoking rustc.

### Requirement: Rust build-script native link metadata binding

r[rust_package_planning.unit_execution.build_script_metadata.link_binding] Mantle MUST bind bounded native link metadata emitted by explicit custom-build host units into downstream Cargo-free rustc target execution.

#### Scenario: Build-script link-search metadata is bound

r[rust_package_planning.unit_execution.build_script_metadata.link_search_binding]

- GIVEN a target unit consumes a custom-build host unit that emits supported `cargo:rustc-link-search` metadata
- WHEN Mantle executes the host-artifact topology rail
- THEN Mantle MUST append deterministic `-L` rustc arguments before invoking the target rustc.

#### Scenario: Build-script link-lib metadata is bound

r[rust_package_planning.unit_execution.build_script_metadata.link_lib_binding]

- GIVEN a target unit consumes a custom-build host unit that emits supported `cargo:rustc-link-lib` metadata
- WHEN Mantle executes the host-artifact topology rail
- THEN Mantle MUST append deterministic `-l` rustc arguments before invoking the target rustc.

#### Scenario: Unsupported link metadata blocks fail closed

r[rust_package_planning.unit_execution.build_script_metadata.link_metadata_blockers]

- GIVEN a custom-build host unit emits malformed, ambiguous, or unsupported native link metadata
- WHEN Mantle parses the build-script metadata
- THEN Mantle MUST return a structured blocker before invoking target rustc.

#### Scenario: Build-script metadata blocks fail closed

r[rust_package_planning.unit_execution.build_script_metadata.blockers]

- GIVEN build-script metadata is required by a target consumer
- WHEN the custom-build executable is missing, fails, emits malformed metadata, or its generated metadata is not available
- THEN Mantle MUST return a structured blocker before invoking target rustc.

### Requirement: Build-script metadata execution receipts

r[rust_package_planning.unit_execution_receipts.build_script_metadata] Mantle MUST emit deterministic JSON evidence for build-script metadata execution.

#### Scenario: CLI JSON includes build-script metadata evidence

r[rust_package_planning.unit_execution_receipts.build_script_metadata.cli]

- GIVEN build-script metadata execution is requested through `rust-plan --execute-host-artifact-topology`
- WHEN Mantle emits the combined JSON receipt
- THEN the receipt MUST include the retained rust plan, ordered host and target unit executions, build-script run metadata, blockers when present, and stable BLAKE3 receipt hashes.

### Requirement: Unified Rust topology execution

r[rust_package_planning.unit_execution.topology] Mantle MUST expose a bounded Cargo-free Rust topology execution rail for supported mixed host and target unit graphs.

#### Scenario: Unified topology CLI executes mixed units

r[rust_package_planning.unit_execution.topology.unified_cli]

- GIVEN a Rust package graph contains supported target units and optional supported host units
- WHEN `rust-plan --execute-topology` is requested with an execution output root
- THEN Mantle MUST emit a combined receipt containing the retained Rust plan and unified topology execution evidence.

#### Scenario: Host and target units execute in explicit order

r[rust_package_planning.unit_execution.topology.mixed_order]

- GIVEN supported host units and supported target units appear in the derivation graph
- WHEN Mantle executes the unified topology rail
- THEN Mantle MUST execute supported host units before target consumers and MUST execute supported target units in dependency order.

#### Scenario: All produced artifact classes are bound

r[rust_package_planning.unit_execution.topology.binds_all_artifacts]

- GIVEN target units consume proc-macro/build-script host artifacts, build-script metadata, or target library artifacts
- WHEN Mantle invokes dependent target rustc units through the unified topology rail
- THEN Mantle MUST bind the produced artifact paths and captured metadata into deterministic rustc inputs before invocation.

#### Scenario: Unsupported or incomplete topology fails closed

r[rust_package_planning.unit_execution.topology.blockers]

- GIVEN the graph is not ready, a required producer is missing, or a unit execution fails
- WHEN Mantle executes the unified topology rail
- THEN Mantle MUST return a structured blocker with ordered partial execution evidence and MUST NOT continue to dependent rustc invocations.

### Requirement: Rust topology output reuse

r[rust_package_planning.unit_execution.topology.output_reuse] Mantle MUST explain rebuild versus reuse for supported Rust topology unit outputs using explicit receipt material.

#### Scenario: Repeated topology execution reuses matching outputs

r[rust_package_planning.unit_execution.topology.output_reuse.repeated]

- GIVEN `rust-plan --execute-topology` has already produced declared output artifacts and per-unit execution receipts under an execution output root
- WHEN the same explicit Rust topology is executed again with matching source digests, toolchain identity, rustc args digest, dependency artifact digests, host artifact digests, declared outputs, and output artifact BLAKE3 digests
- THEN Mantle MUST report successful unit execution with a reuse rebuild reason instead of invoking `rustc` again for that unit.
- AND the reused receipt MUST bind the current output artifact BLAKE3 digests.

### Requirement: Rust topology output reuse blockers

r[rust_package_planning.unit_execution.topology.output_reuse_blockers] Mantle MUST fail closed before invoking `rustc` when prior cached Rust topology output evidence is stale or incomplete.

#### Scenario: Stale cached output blocks reuse

r[rust_package_planning.unit_execution.topology.output_reuse_blockers.stale]

- GIVEN a prior per-unit execution receipt exists under the execution output root
- WHEN a declared output artifact named by that receipt is missing, unreadable, digest-mismatched, or the receipt no longer matches the current explicit unit inputs
- THEN Mantle MUST return a structured stale-cache blocker before invoking `rustc` for that unit.
- AND Mantle MUST NOT silently fall back to Cargo or an unreviewed rebuild for that stale cached unit.

### Requirement: Native Rust unit graph planning fragment

r[rust_package_planning.native_unit_graph_planning] Mantle MUST compute supported Rust unit graph facts from Mantle-owned package, target, and source-closure facts rather than using Cargo unit graph JSON as the source of those facts.

#### Scenario: Supported native unit graph facts match the Cargo oracle

r[rust_package_planning.native_unit_graph_planning.compare]

- GIVEN a tiny local/path Rust workspace is supported by Mantle's native package/target planning fragment
- AND the workspace contains only supported normal `lib` and `bin` targets and path dependencies
- WHEN Mantle plans the Rust unit graph
- THEN Mantle MUST compute native unit identities, target identities, build modes, source inputs, and dependency edges from native Mantle facts.
- AND Mantle MUST compare those native unit graph facts against the retained Cargo unit-graph oracle for the same workspace and options.
- AND the native unit graph fragment MUST be ready only when the supported native facts match the Cargo oracle facts.

#### Scenario: Native unit graph receipts preserve ownership evidence

r[rust_package_planning.native_unit_graph_planning.receipts]

- GIVEN Mantle emits `rust-plan` evidence for a workspace in the supported native unit graph fragment
- WHEN the native unit graph planner finishes
- THEN the receipt MUST include a deterministic native unit graph digest, retained Cargo unit-graph oracle digest, oracle comparison digest, ready flag, and blocker list.
- AND those receipt fields MUST distinguish Mantle-owned unit graph facts from Cargo oracle evidence.

#### Scenario: Ready native unit graph feeds derivation planning

r[rust_package_planning.native_unit_graph_planning.consumes_native]

- GIVEN the native unit graph fragment is ready for a supported workspace
- WHEN Mantle emits `unit_derivation_graph` evidence for supported units
- THEN the derivation graph MUST consume the native unit graph facts rather than Cargo unit graph JSON as the source of unit identities and dependency edges.
- AND the receipt MAY retain Cargo unit graph material only as oracle comparison evidence.

#### Scenario: Unsupported native unit graph inputs fail closed

r[rust_package_planning.native_unit_graph_planning.blockers]

- GIVEN a workspace uses unit graph behavior outside Mantle's supported native fragment
- WHEN Mantle evaluates the native unit graph planner
- THEN Mantle MUST emit deterministic blockers for unsupported target kinds, unsupported unit modes, unsupported feature surfaces, missing or unreadable native package facts, missing source-closure material, unresolved path dependency edges, ambiguous dependency edges, or unsupported dependency kinds.
- AND Mantle MUST NOT silently use Cargo unit graph facts to claim native unit graph readiness.

#### Scenario: Native-vs-oracle mismatch blocks readiness

r[rust_package_planning.native_unit_graph_planning.tests]

- GIVEN Mantle's native unit graph facts disagree with the Cargo unit-graph oracle for a supported comparison surface
- WHEN Mantle records the native unit graph planning receipt
- THEN the receipt MUST set readiness false and include a deterministic mismatch blocker identifying the mismatched surface.
- AND focused tests MUST cover both a supported ready workspace and negative unsupported, missing-edge, and mismatch fixtures.

#### Scenario: Native unit graph planning closes with lifecycle evidence

r[rust_package_planning.native_unit_graph_planning.verify]

- GIVEN the native unit graph planning implementation tasks are complete
- WHEN Mantle accepts the change
- THEN focused Rust verification, Cairn validation, and proposal/design/tasks gates MUST pass before sync, archive, commit, and push.

### Requirement: Native Rust host-unit graph planning fragment

r[rust_package_planning.native_host_unit_graph_planning] Mantle MUST compute supported Rust host-unit graph facts from Mantle-owned package, target, source-closure, and native unit graph facts rather than using Cargo unit graph JSON as the source of those host facts.

#### Scenario: Supported native host-unit graph facts match the Cargo oracle

r[rust_package_planning.native_host_unit_graph_planning.compare]

- GIVEN a tiny local/path Rust workspace is supported by Mantle's native package/target and native unit graph planning fragments
- AND the workspace contains supported `custom-build` or `proc-macro` host units with supported target `lib` or `bin` consumers
- WHEN Mantle plans the Rust host-unit graph
- THEN Mantle MUST compute host unit identities, host execution kinds, declared host artifact classes, generated-metadata placeholder surfaces, and target-consumer edges from native Mantle facts.
- AND Mantle MUST compare those native host-unit graph facts against the retained Cargo unit-graph oracle for the same workspace and options.
- AND the native host-unit graph fragment MUST be ready only when the supported native host facts match the Cargo oracle facts.

#### Scenario: Native host-unit graph receipts preserve ownership evidence

r[rust_package_planning.native_host_unit_graph_planning.receipts]

- GIVEN Mantle emits `rust-plan` evidence for a workspace in the supported native host-unit graph fragment
- WHEN the native host-unit graph planner finishes
- THEN the receipt MUST include a deterministic native host graph digest, retained Cargo host oracle digest, oracle comparison digest, ready flag, blocker list, and self-reference-safe receipt hash.
- AND those receipt fields MUST distinguish Mantle-owned host-unit graph facts from Cargo oracle evidence.

#### Scenario: Ready native host-unit graph feeds derivation planning

r[rust_package_planning.native_host_unit_graph_planning.consumes_native]

- GIVEN the native host-unit graph fragment is ready for a supported workspace
- WHEN Mantle emits `unit_derivation_graph` evidence for supported units
- THEN the derivation graph MUST consume native host-unit graph facts for host nodes, produced host artifacts, generated-metadata placeholders, and target consumer edges.
- AND the receipt MAY retain Cargo unit graph material only as oracle comparison evidence.

#### Scenario: Unsupported native host-unit graph inputs fail closed

r[rust_package_planning.native_host_unit_graph_planning.blockers]

- GIVEN a workspace uses host-unit behavior outside Mantle's supported native fragment
- WHEN Mantle evaluates the native host-unit graph planner
- THEN Mantle MUST emit deterministic blockers for unsupported host target kinds, unsupported unit modes, unsupported feature or profile surfaces, missing native package or source facts, unresolved host target facts, unresolved or ambiguous target-consumer edges, unsupported dependency kinds, or host/target confusion.
- AND Mantle MUST NOT silently use Cargo unit graph facts to claim native host-unit graph readiness.

#### Scenario: Native host-vs-oracle mismatch blocks readiness

r[rust_package_planning.native_host_unit_graph_planning.tests]

- GIVEN Mantle's native host-unit graph facts disagree with the Cargo unit-graph oracle for a supported comparison surface
- WHEN Mantle records the native host-unit graph planning receipt
- THEN the receipt MUST set readiness false and include a deterministic mismatch blocker identifying the mismatched host-unit or consumer-edge surface.
- AND focused tests MUST cover both a supported ready host-unit workspace and negative unsupported, missing-edge, host/target-confused, and mismatch fixtures.

#### Scenario: Native host-unit graph planning closes with lifecycle evidence

r[rust_package_planning.native_host_unit_graph_planning.verify]

- GIVEN the native host-unit graph planning implementation tasks are complete
- WHEN Mantle accepts the change
- THEN focused Rust verification, Cairn validation, and proposal/design/tasks gates MUST pass before sync, archive, commit, and push.

### Requirement: Native Rust host-artifact topology execution

r[rust_package_planning.native_host_artifact_topology_execution] Mantle MUST execute bounded Rust host-artifact topologies from ready Mantle-owned native host-unit graph facts without using Cargo as the build orchestrator.

#### Scenario: Native-planned host artifacts execute before target consumers

r[rust_package_planning.native_host_artifact_topology_execution.executes]

- GIVEN `native_host_unit_graph_planning.ready=true` and `unit_derivation_graph.ready=true`
- AND the graph contains supported `custom-build` or `proc-macro` host derivation nodes plus supported target `lib` or `bin` consumers
- WHEN host-artifact topology execution is requested
- THEN Mantle MUST execute the native-planned host units before their target consumers using explicit derivation args, env, inputs, host artifacts, and declared outputs.
- AND Mantle MUST NOT invoke Cargo as the host or target build orchestrator.

#### Scenario: Produced host artifacts bind into target execution material

r[rust_package_planning.native_host_artifact_topology_execution.binds]

- GIVEN a target unit consumes a host artifact represented by native host-unit graph facts and matching derivation surfaces
- WHEN the producing host unit succeeds
- THEN Mantle MUST bind the produced host artifact path into the target `consumed_host_artifacts`, derivation inputs, and matching `--extern` dependency surfaces before invoking target `rustc`.
- AND the target execution receipt MUST bind the produced host artifact digest.

#### Scenario: Build-script metadata remains deterministic

r[rust_package_planning.native_host_artifact_topology_execution.metadata]

- GIVEN a native-planned `custom-build` host unit emits supported build-script metadata
- WHEN Mantle runs the host-artifact topology rail
- THEN Mantle MUST capture supported metadata surfaces with deterministic `OUT_DIR`, metadata digest, and receipt evidence before target execution.
- AND Mantle MUST bind supported `OUT_DIR`, `rustc-env`, `rustc-cfg`, `rustc-link-lib`, and `rustc-link-search` material into the target execution material before recomputing target argument evidence.

#### Scenario: Unsupported or missing host material blocks before target rustc

r[rust_package_planning.native_host_artifact_topology_execution.blockers]

- GIVEN the native host-unit graph is not ready, a required host producer is absent, a host execution fails, a produced host artifact is stale or unreadable, or host metadata is malformed or unsupported
- WHEN host-artifact topology execution is requested
- THEN Mantle MUST emit a deterministic blocker before invoking the affected target `rustc`.
- AND Mantle MUST NOT repair the missing host material by searching Cargo target directories, registry caches, or ambient git checkouts.

#### Scenario: Bounded execution claim is explicit

r[rust_package_planning.native_host_artifact_topology_execution.bounded_claim]

- GIVEN native host-artifact topology execution succeeds for supported units
- WHEN Mantle reports the result
- THEN the receipt MAY claim bounded Cargo-free host-artifact topology execution for the explicit supported graph only.
- AND the receipt MUST NOT claim full Cargo compatibility, tests/doctests/examples, general scheduling, native-link probing correctness beyond bounded metadata, or bootstrap correctness.

### Requirement: Native Rust unified topology execution

r[rust_package_planning.native_unified_topology_execution] Mantle MUST execute bounded mixed Rust topology graphs from ready Mantle-owned native host-unit graph facts and explicit unit derivation graph evidence without using Cargo as the build orchestrator.

#### Scenario: Native-planned mixed topology executes host units before target consumers

r[rust_package_planning.native_unified_topology_execution.executes]

- GIVEN `native_host_unit_graph_planning.ready=true` and `unit_derivation_graph.ready=true`
- AND the graph contains supported `custom-build` or `proc-macro` host units plus supported target `lib` or `bin` consumers
- WHEN `rust-plan --execute-topology` is requested with an execution output root
- THEN Mantle MUST execute the native-planned host units before affected target consumers using explicit derivation args, env, inputs, host artifacts, metadata, and declared outputs.
- AND Mantle MUST NOT invoke Cargo as the host or target build orchestrator.

#### Scenario: Native host artifacts and metadata bind into unified target execution

r[rust_package_planning.native_unified_topology_execution.binds]

- GIVEN a unified topology target unit consumes proc-macro artifacts, build-script artifacts, build-script metadata, or target library artifacts
- WHEN the producing units succeed
- THEN Mantle MUST bind produced host artifact paths, captured build-script metadata, and produced target library artifact paths into deterministic target `rustc` inputs before invocation.
- AND the target execution receipts MUST bind the produced artifact and metadata digests.

#### Scenario: Unsupported or missing native host material blocks unified topology execution

r[rust_package_planning.native_unified_topology_execution.blockers]

- GIVEN the native host-unit graph is not ready, a required native host producer is absent, a host derivation is not backed by native host facts, a host artifact consumer is not backed by native host facts, a host execution fails, a produced host artifact is stale or unreadable, or host metadata is malformed or unsupported
- WHEN `rust-plan --execute-topology` is requested
- THEN Mantle MUST emit a deterministic blocker before invoking the affected host or target `rustc`.
- AND Mantle MUST NOT repair the missing host material by searching Cargo target directories, registry caches, or ambient git checkouts.

#### Scenario: Unified topology receipts preserve native-host evidence

r[rust_package_planning.native_unified_topology_execution.receipts]

- GIVEN native unified topology execution succeeds or fails closed for a supported graph boundary
- WHEN Mantle emits the combined JSON receipt
- THEN the receipt MUST include the retained Rust plan, ordered unit execution receipts, build-script metadata runs when present, blockers when present, a bounded native-host unified-topology claim, and stable BLAKE3 receipt hashes.
- AND those receipt fields MUST distinguish Mantle-owned native host graph evidence from retained Cargo oracle evidence.

#### Scenario: Native unified topology behavior is covered by focused fixtures

r[rust_package_planning.native_unified_topology_execution.tests]

- GIVEN Mantle includes focused `rust_plan_cli` fixtures for unified topology execution
- WHEN tests exercise mixed proc-macro or build-script topologies and blocked native host graph inputs
- THEN the positive fixtures MUST prove successful native-backed mixed topology execution.
- AND the negative fixtures MUST prove blocked native host graph material yields a deterministic pre-execution blocker and zero affected target executions.

#### Scenario: Native unified topology change closes with lifecycle evidence

r[rust_package_planning.native_unified_topology_execution.verify]

- GIVEN the native unified topology execution implementation tasks are complete
- WHEN Mantle accepts the change
- THEN focused Rust verification, Cairn validation, and proposal/design/tasks gates MUST pass before sync, archive, commit, and push.

### Requirement: Native Rust registry source planning fragment

r[rust_package_planning.native_registry_source] Mantle MUST represent supported registry-backed Rust package sources as explicit native source facts before those packages participate in Cargo-free native unit or topology claims.

#### Scenario: Lockfile registry identities are recorded

r[rust_package_planning.native_registry_source.lockfile_identity]

- GIVEN a Cargo lockfile contains a supported registry package entry
- WHEN Mantle computes native Rust source-planning facts
- THEN Mantle MUST record the package name, version, source URL/class, checksum material, lockfile digest, and package identity used by downstream unit graph evidence.
- AND missing checksum or unsupported lockfile source material MUST make the native registry source fragment not ready.

#### Scenario: Declared vendor source roots are digest-bound

r[rust_package_planning.native_registry_source.vendor_digest]

- GIVEN a supported registry package has declared local vendor/source material
- WHEN Mantle computes native Rust source-planning facts
- THEN Mantle MUST bind the package to a deterministic source-root reference and BLAKE3 source-tree digest.
- AND Mantle MUST NOT read `$CARGO_HOME`, Cargo registry caches, Cargo git checkouts, target directories, or network locations as undeclared source material.

#### Scenario: Native registry source facts are compared with Cargo oracle material

r[rust_package_planning.native_registry_source.oracle_compare]

- GIVEN Cargo oracle material is retained for the same workspace and lockfile
- WHEN Mantle supports a registry package in the native source-planning fragment
- THEN Mantle MUST compare native package identity, source class, checksum material, and package/source membership against the Cargo oracle evidence.
- AND native-vs-oracle divergence MUST produce deterministic blockers instead of silently falling back to Cargo-derived source paths.

#### Scenario: Unsupported or stale registry source material fails closed

r[rust_package_planning.native_registry_source.blockers]

- GIVEN a package source is registry-backed or vendor-backed
- WHEN the lockfile checksum is missing, the vendor/source root is missing or unreadable, the source digest mismatches recorded material, the source kind/layout is unsupported, or required source material would come from an ambient Cargo cache
- THEN Mantle MUST emit deterministic native registry source blockers before claiming native unit graph readiness or Cargo-free execution for the affected package.

#### Scenario: Registry source facts are receipt-bound

r[rust_package_planning.native_registry_source.receipts]

- GIVEN Mantle emits `rust-plan` JSON evidence for a workspace with supported registry source material
- WHEN native registry source planning finishes
- THEN the receipt MUST include ready status, ordered registry source facts, lockfile/source digests, oracle comparison evidence, blockers when present, and a stable receipt hash.
- AND the receipt MUST keep the bounded claim explicit: declared local/vendor source planning only, with no network fetch, version solving, remote cache, or general Cargo registry compatibility claim.

#### Scenario: Registry source planning is covered by focused fixtures

r[rust_package_planning.native_registry_source.tests]

- GIVEN Mantle includes focused Rust planner or CLI fixtures for registry source planning
- WHEN tests exercise a supported vendored registry package and missing or stale vendor/source material
- THEN positive fixtures MUST prove ready native source facts with explicit BLAKE3-bound source material.
- AND negative fixtures MUST prove deterministic fail-closed blockers without consulting ambient Cargo caches.

#### Scenario: Registry source planning change closes with lifecycle evidence

r[rust_package_planning.native_registry_source.verify]

- GIVEN the native registry source planning implementation tasks are complete
- WHEN Mantle accepts the change
- THEN focused Rust verification, Cairn validation, and proposal/design/tasks gates MUST pass before sync, archive, commit, and push.

### Requirement: Native Rust registry topology execution

r[rust_package_planning.native_registry_topology_execution] Mantle MUST execute bounded Rust topology graphs containing supported registry-backed packages only from ready native registry source facts and explicit unit derivation graph evidence, without using Cargo as the build orchestrator.

#### Scenario: Registry source facts gate native topology execution

r[rust_package_planning.native_registry_topology_execution.source_facts]

- GIVEN `rust-plan --execute-topology` is requested for a graph containing a registry-backed package
- WHEN Mantle evaluates whether that package can participate in native topology execution
- THEN Mantle MUST require a ready `native_registry_source_planning` fact for the package, including lockfile identity, checksum material, declared vendor/source root, source digest, and oracle comparison evidence.
- AND Mantle MUST NOT treat Cargo registry cache paths, `$CARGO_HOME`, target directories, git checkouts, or network locations as substitute source facts.

#### Scenario: Vendored registry dependency executes through explicit topology evidence

r[rust_package_planning.native_registry_topology_execution.executes]

- GIVEN a local/path root package depends on a supported vendored registry-backed `lib` package
- AND the registry package has ready native registry source facts and a supported explicit unit derivation node
- WHEN topology execution is requested with an explicit execution output root
- THEN Mantle MUST execute the registry-backed producer before affected consumers using only explicit derivation args, env, source material, dependency artifacts, host artifacts, and declared outputs.
- AND Mantle MUST bind the produced registry-backed artifact into downstream target execution before invoking the consumer `rustc`.
- AND Mantle MUST NOT invoke Cargo as the producer or consumer build orchestrator.

#### Scenario: Registry topology receipts bind source and artifact evidence

r[rust_package_planning.native_registry_topology_execution.receipts]

- GIVEN a registry-backed topology execution succeeds or fails closed
- WHEN Mantle emits the combined CLI JSON receipt
- THEN the receipt MUST preserve the retained `rust_plan.native_registry_source_planning` evidence and ordered topology execution receipts.
- AND executed registry-backed unit receipts MUST bind lockfile/source identity, source digest, rustc argument digest, produced artifact BLAKE3 digests, and stable receipt hashes.
- AND the bounded claim MUST identify declared local/vendor registry source execution only, not general Cargo registry compatibility.

#### Scenario: Unsupported or stale registry source material blocks before rustc

r[rust_package_planning.native_registry_topology_execution.blockers]

- GIVEN a topology graph contains a registry-backed package whose lockfile checksum is missing, vendor/source root is missing or unreadable, vendored package material is absent, source digest or oracle material mismatches, source layout is unsupported, or required material would come from an ambient Cargo cache
- WHEN topology execution is requested
- THEN Mantle MUST emit a deterministic native registry topology blocker before invoking `rustc` for the affected registry-backed unit or its consumers.
- AND the receipt MUST show zero successful executions for affected units whose required registry source material was not ready.

#### Scenario: Registry topology execution is covered by focused CLI fixtures

r[rust_package_planning.native_registry_topology_execution.tests]

- GIVEN Mantle includes focused `rust_plan_cli` fixtures for registry-backed topology execution
- WHEN tests exercise a supported vendored registry-backed dependency and a missing or stale vendor source case
- THEN the positive fixture MUST prove successful topology execution with registry source facts consumed by execution receipts.
- AND the negative fixture MUST prove deterministic fail-closed blockers without consulting ambient Cargo caches.

#### Scenario: Registry topology execution change closes with lifecycle evidence

r[rust_package_planning.native_registry_topology_execution.verify]

- GIVEN the native registry topology execution implementation tasks are complete
- WHEN Mantle accepts the change
- THEN focused Rust verification, Cairn validation, and proposal/design/tasks gates MUST pass before sync, archive, commit, and push.

### Requirement: Native Rust registry host-artifact topology execution

r[rust_package_planning.native_registry_host_artifact_topology_execution] Mantle MUST execute bounded Rust host-artifact topology graphs containing supported registry-backed packages only from ready native registry source facts, ready native host-unit graph facts, and explicit unit derivation graph evidence, without using Cargo as the build orchestrator.

#### Scenario: Registry source and native host facts gate host-artifact execution

r[rust_package_planning.native_registry_host_artifact_topology_execution.source_and_host_facts]

- GIVEN `rust-plan --execute-topology` or host-artifact topology execution is requested for a graph containing a registry-backed `custom-build` or `proc-macro` host package
- WHEN Mantle evaluates whether that host package can participate in native topology execution
- THEN Mantle MUST require ready `native_registry_source_planning` facts for the package, including lockfile identity, checksum material, declared vendor/source root, source digest, and oracle comparison evidence.
- AND Mantle MUST require ready `native_host_unit_graph_planning` facts for the host producer and target consumer relationship.
- AND Mantle MUST NOT treat Cargo registry cache paths, `$CARGO_HOME`, target directories, git checkouts, or network locations as substitute source or host facts.

#### Scenario: Vendored registry host artifact executes through explicit topology evidence

r[rust_package_planning.native_registry_host_artifact_topology_execution.executes]

- GIVEN a local/path root package depends on a supported vendored registry-backed package that provides a `proc-macro` or `custom-build` host unit
- AND the registry host package has ready native registry source facts, ready native host-unit graph facts, and supported explicit unit derivation nodes
- WHEN topology execution is requested with an explicit execution output root
- THEN Mantle MUST execute the registry-backed host producer before affected target consumers using only explicit derivation args, env, source material, dependency artifacts, host artifacts, build-script metadata, and declared outputs.
- AND Mantle MUST bind produced registry-backed host artifacts or build-script metadata into downstream target execution before invoking consumer `rustc`.
- AND Mantle MUST NOT invoke Cargo as the producer, build-script, proc-macro, or consumer build orchestrator.

#### Scenario: Registry host-artifact receipts bind source, host, metadata, and target evidence

r[rust_package_planning.native_registry_host_artifact_topology_execution.receipts]

- GIVEN a registry-backed host-artifact topology execution succeeds or fails closed
- WHEN Mantle emits the combined CLI JSON receipt
- THEN the receipt MUST preserve retained `rust_plan.native_registry_source_planning`, `rust_plan.native_host_unit_graph_planning`, and ordered topology execution evidence.
- AND executed registry-backed host-unit receipts MUST bind lockfile/source identity, source digest, rustc argument digest, produced host artifact BLAKE3 digests, build-script metadata digests when applicable, target consumer artifact digests, and stable receipt hashes.
- AND the bounded claim MUST identify declared local/vendor registry host-artifact execution only, not general Cargo registry compatibility.

#### Scenario: Unsupported or stale registry host material blocks before execution

r[rust_package_planning.native_registry_host_artifact_topology_execution.blockers]

- GIVEN a topology graph contains a registry-backed host package whose lockfile checksum is missing, vendor/source root is missing or unreadable, vendored package material is absent, source digest or oracle material mismatches, host graph evidence is missing or mismatched, build-script metadata is malformed or unsupported, produced host artifacts are missing or stale, or required material would come from an ambient Cargo cache
- WHEN topology execution is requested
- THEN Mantle MUST emit a deterministic native registry host-artifact topology blocker before invoking `rustc`, a build-script executable, or an affected target consumer.
- AND the receipt MUST show zero successful executions for affected units whose required registry source or host material was not ready.

#### Scenario: Registry host-artifact topology execution is covered by focused CLI fixtures

r[rust_package_planning.native_registry_host_artifact_topology_execution.tests]

- GIVEN Mantle includes focused `rust_plan_cli` fixtures for registry-backed host-artifact topology execution
- WHEN tests exercise a supported vendored registry-backed build-script or proc-macro dependency and a missing or stale vendor source case
- THEN the positive fixture MUST prove successful topology execution with registry source facts and native host facts consumed by execution receipts.
- AND the negative fixture MUST prove deterministic fail-closed blockers without consulting ambient Cargo caches.

#### Scenario: Registry host-artifact topology execution change closes with lifecycle evidence

r[rust_package_planning.native_registry_host_artifact_topology_execution.verify]

- GIVEN the native registry host-artifact topology execution implementation tasks are complete
- WHEN Mantle accepts the change
- THEN focused Rust verification, Cairn validation, and proposal/design/tasks gates MUST pass before sync, archive, commit, and push.
