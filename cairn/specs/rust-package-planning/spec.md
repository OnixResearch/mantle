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

### Requirement: Native Rust registry unified host topology execution

r[rust_package_planning.native_registry_unified_host_topology_execution] Mantle MUST execute bounded unified Rust topology graphs containing supported registry-backed host producers only from ready native registry source facts, ready native host-unit graph facts, and explicit unit derivation graph evidence, without using Cargo as the build orchestrator.

#### Scenario: Registry source and native host facts gate unified host execution

r[rust_package_planning.native_registry_unified_host_topology_execution.source_and_host_facts]

- GIVEN `rust-plan --execute-topology` is requested for a graph containing a registry-backed `custom-build` or `proc-macro` host package
- WHEN Mantle evaluates whether that host package or an affected target consumer can execute in the unified topology
- THEN Mantle MUST require ready `native_registry_source_planning` facts for the registry host package, including lockfile identity, checksum material, declared vendor/source root, source digest, and oracle comparison evidence.
- AND Mantle MUST require ready `native_host_unit_graph_planning` facts for the host producer and target consumer relationship.
- AND Mantle MUST NOT treat Cargo registry cache paths, `$CARGO_HOME`, target directories, git checkouts, or network locations as substitute source or host facts.

#### Scenario: Vendored registry host producer executes through unified topology evidence

r[rust_package_planning.native_registry_unified_host_topology_execution.executes]

- GIVEN a local/path root package depends on a supported vendored registry-backed package that provides a `proc-macro` or `custom-build` host unit
- AND the registry host package has ready native registry source facts, ready native host-unit graph facts, and supported explicit unit derivation nodes
- WHEN unified topology execution is requested with an explicit execution output root
- THEN Mantle MUST execute the registry-backed host producer before affected target consumers using only explicit derivation args, env, source material, dependency artifacts, host artifacts, build-script metadata, and declared outputs.
- AND Mantle MUST bind produced registry-backed host artifacts or build-script metadata into downstream target execution before invoking consumer `rustc`.
- AND Mantle MUST NOT invoke Cargo as the producer, build-script, proc-macro, or consumer build orchestrator.

#### Scenario: Unified registry host topology receipts bind source, host, metadata, and target evidence

r[rust_package_planning.native_registry_unified_host_topology_execution.receipts]

- GIVEN unified topology execution containing a registry-backed host producer succeeds or fails closed
- WHEN Mantle emits the combined CLI JSON receipt
- THEN the receipt MUST preserve retained `rust_plan.native_registry_source_planning`, `rust_plan.native_host_unit_graph_planning`, `rust_plan.unit_derivation_graph`, and ordered `topology_execution` evidence.
- AND executed registry-backed host-unit receipts MUST bind lockfile/source identity, source digest, rustc argument digest, produced host artifact BLAKE3 digests, and build-script metadata digests when applicable.
- AND target consumer receipts MUST bind consumed host artifact or build-script metadata evidence and target output BLAKE3 digests.
- AND the bounded claim MUST identify declared local/vendor registry unified host topology execution only, not general Cargo registry compatibility.

#### Scenario: Unsupported or stale registry host material blocks before unified execution

r[rust_package_planning.native_registry_unified_host_topology_execution.blockers]

- GIVEN a unified topology graph contains a registry-backed host package whose lockfile checksum is missing, vendor/source root is missing or unreadable, vendored package material is absent, source digest or oracle material mismatches, host graph evidence is missing or mismatched, build-script metadata is malformed or unsupported, produced host artifacts are missing or stale, or required material would come from an ambient Cargo cache
- WHEN unified topology execution is requested
- THEN Mantle MUST emit a deterministic native registry unified host topology blocker before invoking `rustc`, a build-script executable, or an affected target consumer.
- AND the receipt MUST show zero successful executions for affected units whose required registry source or host material was not ready.

#### Scenario: Registry unified host topology execution is covered by focused CLI fixtures

r[rust_package_planning.native_registry_unified_host_topology_execution.tests]

- GIVEN Mantle includes focused `rust_plan_cli` fixtures for registry-backed unified host topology execution
- WHEN tests exercise a supported vendored registry-backed build-script or proc-macro dependency and a missing or stale vendor host-source case through `--execute-topology`
- THEN the positive fixture MUST prove successful unified topology execution with registry source facts and native host facts consumed by execution receipts.
- AND the negative fixture MUST prove deterministic fail-closed blockers without consulting ambient Cargo caches.

#### Scenario: Registry unified host topology execution change closes with lifecycle evidence

r[rust_package_planning.native_registry_unified_host_topology_execution.verify]

- GIVEN the native registry unified host topology execution implementation tasks are complete
- WHEN Mantle accepts the change
- THEN focused Rust verification, Cairn validation, and proposal/design/tasks gates MUST pass before sync, archive, commit, and push.

# Rust package planning spec delta

## ADDED Requirements

### r[rust_package_planning.native_registry_proc_macro_unified_topology]

Mantle MUST execute bounded unified Rust topology graphs containing supported registry-backed proc-macro host producers only from ready native registry source facts, ready native host-unit graph facts, and explicit unit derivation graph evidence, without using Cargo as the build orchestrator.

#### Scenario: Registry proc-macro source and host facts gate unified execution

- **GIVEN** a local Rust target depends on a vendored registry proc-macro package
- **AND** the package is declared by `Cargo.lock` identity, checksum, and supported local vendor source facts
- **AND** native host-unit graph planning contains a matching proc-macro host unit and target-consumer edge
- **WHEN** Mantle runs `rust-plan --execute-topology`
- **THEN** the proc-macro host producer is eligible for execution only after all native registry source, native host graph, and unit derivation graph facts are ready
- **AND** Cargo is retained only as oracle/evidence, not as the execution orchestrator.

#### Scenario: Registry proc-macro host producer executes before target consumers

- **GIVEN** the registry proc-macro host producer and target consumer facts are ready
- **WHEN** Mantle executes the unified topology
- **THEN** the proc-macro host producer executes before the consuming target unit
- **AND** the consuming target unit receives the produced proc-macro host artifact through explicit derivation input and receipt material.

#### Scenario: Registry proc-macro receipts bind source and artifact identity

- **GIVEN** a registry proc-macro host artifact is produced by unified topology execution
- **WHEN** Mantle emits the JSON receipt
- **THEN** the receipt binds the registry package identity, checksum/source fact, source digest, proc-macro artifact digest, target-consumer artifact use, and declared output digest evidence
- **AND** those bindings use deterministic BLAKE3 material where Mantle owns the digest surface.

#### Scenario: Unsupported registry proc-macro material blocks before rustc

- **GIVEN** the registry proc-macro source layout is unsupported, missing, stale, or would require ambient Cargo registry cache material
- **WHEN** Mantle runs unified topology execution
- **THEN** execution is blocked before invoking `rustc` for the proc-macro or consuming target
- **AND** the JSON receipt records deterministic blocker reasons and zero successful unit executions for that topology.

#### Scenario: Registry proc-macro unified topology does not claim broad Cargo registry compatibility

- **GIVEN** a registry proc-macro package requires unsupported Cargo registry behavior, version solving, network/index access, `$CARGO_HOME`, ambient cache lookup, unsupported build-script/native-link probing, or unsupported proc-macro surfaces
- **WHEN** Mantle plans or executes the topology
- **THEN** Mantle MUST fail closed with explicit blockers rather than silently using Cargo or ambient state.

#### Scenario: CLI evidence covers registry proc-macro unified topology

- **GIVEN** the implementation is complete
- **WHEN** the test suite runs
- **THEN** positive CLI JSON coverage proves vendored registry proc-macro unified topology execution
- **AND** negative CLI JSON coverage proves unsupported/missing registry proc-macro material blocks before `rustc`.

# Rust package planning spec delta

## ADDED Requirements

### r[rust_package_planning.native_registry_transitive_topology_execution]

Mantle MUST execute bounded native Rust topology graphs containing transitive vendored-registry dependency chains only from ready native registry source facts and explicit unit derivation graph evidence, without using Cargo as the build orchestrator, network/index access, `$CARGO_HOME`, ambient registry caches, or version solving.

#### Scenario: Ready transitive registry source facts gate execution

Given a local Rust package depends on vendored registry package A
And vendored registry package A depends on vendored registry package B
And the lockfile and declared vendor source replacement provide supported source facts for both registry packages
When Mantle plans and executes the topology
Then `native_registry_source_planning.ready` is true
And the receipt records ready source facts for registry package A and registry package B
And execution uses those ready source facts rather than ambient Cargo cache material.

#### Scenario: Producer-first transitive topology execution

Given a supported topology `local app -> registry package A -> registry package B`
When Mantle executes the topology
Then registry package B is executed before registry package A
And registry package A is executed before the local app
And each execution receipt records deterministic source, rustc argument, output artifact, and receipt hash evidence.

#### Scenario: Transitive registry dependency artifacts are bound into consumers

Given registry package B produces a library artifact consumed by registry package A
And registry package A produces a library artifact consumed by the local app
When Mantle executes the topology
Then registry package A's execution receipt records dependency artifact digest evidence for registry package B
And the local app's execution receipt records dependency artifact digest evidence for registry package A
And downstream `rustc` material is rewritten only from explicit producer artifacts.

#### Scenario: Missing or unsupported transitive vendor material blocks before rustc

Given a registry package in the transitive closure has missing, stale, or unsupported vendored source material
When Mantle is asked to execute the topology
Then Mantle reports deterministic native-registry blockers before executing affected downstream `rustc` commands
And the receipt does not claim fallback to Cargo orchestration, network/index access, `$CARGO_HOME`, ambient registry caches, or version solving.

# Rust package planning spec delta

## ADDED Requirements

### r[rust_package_planning.native_registry_feature_gated_topology_planning]

Mantle MUST plan bounded native Rust topology graphs containing feature-selected vendored-registry dependency surfaces only from explicit native feature facts, ready native registry source facts, and explicit unit derivation graph evidence, without using Cargo as the build orchestrator, Cargo resolver fallback, network/index access, `$CARGO_HOME`, ambient registry caches, or version solving.

#### Scenario: Explicit selected feature facts activate a bounded optional registry dependency

Given a local Rust package selects a supported feature on a vendored registry package
And that selected feature activates a supported optional vendored registry dependency
When Mantle plans the Rust package graph
Then Mantle records deterministic selected feature facts
And Mantle records the activated optional registry dependency in native planning evidence
And Mantle requires ready native registry source facts for every activated registry package.

#### Scenario: Feature-selected registry topology executes from native facts

Given the selected feature graph is within Mantle's supported bounded feature surface
And every activated registry package has ready native registry source facts
When Mantle executes the topology
Then only selected feature graph units enter execution
And unselected optional registry packages do not execute
And downstream `rustc` material binds selected optional dependency artifacts through explicit topology receipts.

#### Scenario: Unsupported feature behavior blocks before rustc

Given a registry feature surface requires unsupported default-feature behavior, workspace feature inheritance, target-specific feature activation, ambiguous feature names, or resolver-dependent feature unification
When Mantle plans or executes the topology
Then Mantle reports deterministic feature-surface blockers before executing affected downstream `rustc` commands
And the receipt does not claim fallback to Cargo resolver behavior, Cargo orchestration, network/index access, `$CARGO_HOME`, ambient registry caches, or version solving.

#### Scenario: CLI coverage proves positive and negative feature seams

Given Mantle has positive and negative CLI fixtures for feature-gated vendored registry topologies
When Mantle's CLI test suite runs
Then positive CLI JSON coverage proves explicit feature-selected vendored registry topology planning and execution
And negative CLI JSON coverage proves unsupported feature surfaces block before `rustc`.

# Rust package planning spec delta

## ADDED Requirements

### r[rust_package_planning.native_registry_target_cfg_topology_planning]

Mantle MUST plan bounded native Rust topology graphs containing target-cfg-selected vendored-registry dependency surfaces only from explicit native target-cfg facts, ready native registry source facts, and explicit unit derivation graph evidence, without using Cargo as the build orchestrator, Cargo resolver fallback, network/index access, `$CARGO_HOME`, ambient registry caches, or version solving.

#### Scenario: Supported target-cfg facts select a registry dependency

Given a vendored registry package declares a dependency under a supported `target.'cfg(...)'.dependencies` table
And Mantle can decide that cfg predicate from an explicit active Rust target triple
When Mantle plans the Rust package graph
Then Mantle records deterministic target-cfg selection evidence
And Mantle records the selected registry dependency in native planning evidence
And Mantle requires ready native registry source facts for every selected registry package.

#### Scenario: Unselected target-cfg dependency does not execute

Given a vendored registry package declares a dependency under a supported target-cfg table
And the active Rust target triple does not select that cfg predicate
When Mantle plans and executes the topology
Then the unselected target-cfg registry package does not enter execution
And downstream `rustc` material does not bind an artifact for that unselected dependency.

#### Scenario: Target-cfg-selected registry topology executes from native facts

Given the target-cfg graph is within Mantle's supported bounded cfg surface
And every selected registry package has ready native registry source facts
When Mantle executes the topology
Then selected target-cfg graph units enter execution in producer-first order
And downstream `rustc` material binds selected target-cfg dependency artifacts through explicit topology receipts.

#### Scenario: Unsupported target-cfg behavior blocks before rustc

Given a registry target-cfg surface requires unsupported cfg expressions, target-specific feature activation, ambiguous platform selection, resolver-dependent behavior, or Cargo platform matching beyond Mantle's bounded evaluator
When Mantle plans or executes the topology
Then Mantle reports deterministic target-cfg blockers before executing affected downstream `rustc` commands
And the receipt does not claim fallback to Cargo resolver behavior, Cargo orchestration, network/index access, `$CARGO_HOME`, ambient registry caches, or version solving.

#### Scenario: CLI coverage proves positive and negative target-cfg seams

Given Mantle has positive and negative CLI fixtures for target-cfg vendored registry topologies
When Mantle's CLI test suite runs
Then positive CLI JSON coverage proves explicit target-cfg-selected vendored registry topology planning and execution
And negative CLI JSON coverage proves unsupported target-cfg surfaces block before `rustc`.

### r[rust_package_planning.native_registry_workspace_dependency_topology_planning]

Mantle MUST plan bounded native Rust topology graphs containing workspace-inherited vendored-registry dependency surfaces only from explicit root workspace dependency facts, member inheritance facts, ready native registry source facts, and explicit unit derivation graph evidence, without using Cargo as the build orchestrator, Cargo resolver fallback, network/index access, `$CARGO_HOME`, ambient registry caches, or version solving.

#### Scenario: Workspace dependency facts select an inherited registry dependency

Given a workspace root declares a supported vendored-registry dependency in `[workspace.dependencies]`
And a workspace member declares that dependency with `{ workspace = true }`
And the inherited dependency is within Mantle's bounded supported fragment
When Mantle plans the Rust package graph
Then Mantle records deterministic workspace dependency inheritance evidence
And Mantle records the inherited registry dependency in native planning evidence
And Mantle requires ready native registry source facts for every inherited registry package.

#### Scenario: Workspace-inherited registry topology executes from native facts

Given the workspace dependency inheritance graph is within Mantle's supported bounded surface
And every inherited registry package has ready native registry source facts
When Mantle executes the topology
Then inherited workspace dependency graph units enter execution in producer-first order
And downstream `rustc` material binds inherited dependency artifacts through explicit topology receipts.

#### Scenario: Unsupported workspace dependency inheritance blocks before rustc

Given a workspace dependency surface requires missing root entries, unsupported inherited feature/default-feature behavior, target-specific inheritance beyond Mantle's bounded evaluator, ambiguous package renames, resolver-dependent behavior, or Cargo workspace matching beyond Mantle's bounded evaluator
When Mantle plans or executes the topology
Then Mantle reports deterministic workspace dependency blockers before executing affected downstream `rustc` commands
And the receipt does not claim fallback to Cargo resolver behavior, Cargo orchestration, network/index access, `$CARGO_HOME`, ambient registry caches, or version solving.

#### Scenario: CLI coverage proves positive and negative workspace dependency seams

Given Mantle has positive and negative CLI fixtures for workspace-inherited vendored registry topologies
When Mantle's CLI test suite runs
Then positive CLI JSON coverage proves explicit workspace-inherited vendored registry topology planning and execution
And negative CLI JSON coverage proves unsupported workspace dependency inheritance surfaces block before `rustc`.

# Rust package planning spec delta

## ADDED Requirements

### r[rust_package_planning.native_registry_build_dependency_topology_planning]

Mantle MUST represent `native_registry_build_dependency_topology_planning` as bounded native Rust planning evidence and MUST emit deterministic blockers before execution when the surface would require unsupported Cargo behavior.

#### Scenario: Supported bounded surface is planned from native facts

- GIVEN a Rust workspace declares the bounded surface for `native_registry_build_dependency_topology_planning`
- AND all required local or vendored-registry source facts are ready
- WHEN Mantle runs `rust-plan` for the workspace
- THEN Mantle MUST emit native receipt evidence identifying the selected package/member/dependency facts and source material.
- AND Mantle MUST NOT require Cargo orchestration, network access, `$CARGO_HOME`, registry cache fallback, or lockfile mutation for the native claim.

#### Scenario: Supported topology binds selected artifacts

- GIVEN native planning for `native_registry_build_dependency_topology_planning` is ready
- AND the selected package or dependency edge participates in a supported topology rail
- WHEN Mantle executes the topology
- THEN producer artifacts MUST be built before consumers and bound by declared output digest evidence.
- AND the topology receipt MUST expose enough evidence to review the dependency source and artifact binding.

#### Scenario: Unsupported behavior blocks before rustc

- GIVEN a Rust workspace declares a shape for `native_registry_build_dependency_topology_planning` that is outside Mantle's bounded native fragment
- WHEN Mantle plans or executes the topology
- THEN Mantle MUST fail closed with a deterministic blocker identifying the unsupported class before invoking `rustc` for the affected claim.
- AND Mantle MUST NOT silently fall back to Cargo resolver behavior or ambient registry/cache material.

#### Scenario: CLI coverage proves positive and negative behavior

- GIVEN the implementation claims support for `native_registry_build_dependency_topology_planning`
- WHEN the relevant `rust_plan` or `rust_plan_cli` tests run
- THEN they MUST include at least one supported fixture and one unsupported fixture with deterministic assertions.

# Rust package planning spec delta

## ADDED Requirements

### r[rust_package_planning.native_registry_workspace_dependency_feature_inheritance]

Mantle MUST represent `native_registry_workspace_dependency_feature_inheritance` as bounded native Rust planning evidence and MUST emit deterministic blockers before execution when the surface would require unsupported Cargo behavior.

#### Scenario: Supported bounded surface is planned from native facts

- GIVEN a Rust workspace declares the bounded surface for `native_registry_workspace_dependency_feature_inheritance`
- AND all required local or vendored-registry source facts are ready
- WHEN Mantle runs `rust-plan` for the workspace
- THEN Mantle MUST emit native receipt evidence identifying the selected package/member/dependency facts and source material.
- AND Mantle MUST NOT require Cargo orchestration, network access, `$CARGO_HOME`, registry cache fallback, or lockfile mutation for the native claim.

#### Scenario: Supported topology binds selected artifacts

- GIVEN native planning for `native_registry_workspace_dependency_feature_inheritance` is ready
- AND the selected package or dependency edge participates in a supported topology rail
- WHEN Mantle executes the topology
- THEN producer artifacts MUST be built before consumers and bound by declared output digest evidence.
- AND the topology receipt MUST expose enough evidence to review the dependency source and artifact binding.

#### Scenario: Unsupported behavior blocks before rustc

- GIVEN a Rust workspace declares a shape for `native_registry_workspace_dependency_feature_inheritance` that is outside Mantle's bounded native fragment
- WHEN Mantle plans or executes the topology
- THEN Mantle MUST fail closed with a deterministic blocker identifying the unsupported class before invoking `rustc` for the affected claim.
- AND Mantle MUST NOT silently fall back to Cargo resolver behavior or ambient registry/cache material.

#### Scenario: CLI coverage proves positive and negative behavior

- GIVEN the implementation claims support for `native_registry_workspace_dependency_feature_inheritance`
- WHEN the relevant `rust_plan` or `rust_plan_cli` tests run
- THEN they MUST include at least one supported fixture and one unsupported fixture with deterministic assertions.

# Rust package planning spec delta

## ADDED Requirements

### r[rust_package_planning.native_registry_patch_source_planning]

Mantle MUST represent `native_registry_patch_source_planning` as bounded native Rust planning evidence and MUST emit deterministic blockers before execution when the surface would require unsupported Cargo behavior.

#### Scenario: Supported bounded surface is planned from native facts

- GIVEN a Rust workspace declares the bounded surface for `native_registry_patch_source_planning`
- AND all required local or vendored-registry source facts are ready
- WHEN Mantle runs `rust-plan` for the workspace
- THEN Mantle MUST emit native receipt evidence identifying the selected package/member/dependency facts and source material.
- AND Mantle MUST NOT require Cargo orchestration, network access, `$CARGO_HOME`, registry cache fallback, or lockfile mutation for the native claim.

#### Scenario: Supported topology binds selected artifacts

- GIVEN native planning for `native_registry_patch_source_planning` is ready
- AND the selected package or dependency edge participates in a supported topology rail
- WHEN Mantle executes the topology
- THEN producer artifacts MUST be built before consumers and bound by declared output digest evidence.
- AND the topology receipt MUST expose enough evidence to review the dependency source and artifact binding.

#### Scenario: Unsupported behavior blocks before rustc

- GIVEN a Rust workspace declares a shape for `native_registry_patch_source_planning` that is outside Mantle's bounded native fragment
- WHEN Mantle plans or executes the topology
- THEN Mantle MUST fail closed with a deterministic blocker identifying the unsupported class before invoking `rustc` for the affected claim.
- AND Mantle MUST NOT silently fall back to Cargo resolver behavior or ambient registry/cache material.

#### Scenario: CLI coverage proves positive and negative behavior

- GIVEN the implementation claims support for `native_registry_patch_source_planning`
- WHEN the relevant `rust_plan` or `rust_plan_cli` tests run
- THEN they MUST include at least one supported fixture and one unsupported fixture with deterministic assertions.

# Rust package planning spec delta

## ADDED Requirements

### r[rust_package_planning.native_rust_workspace_member_glob_planning]

Mantle MUST represent `native_rust_workspace_member_glob_planning` as bounded native Rust planning evidence and MUST emit deterministic blockers before execution when the surface would require unsupported Cargo behavior.

#### Scenario: Supported bounded surface is planned from native facts

- GIVEN a Rust workspace declares the bounded surface for `native_rust_workspace_member_glob_planning`
- AND all required local or vendored-registry source facts are ready
- WHEN Mantle runs `rust-plan` for the workspace
- THEN Mantle MUST emit native receipt evidence identifying the selected package/member/dependency facts and source material.
- AND Mantle MUST NOT require Cargo orchestration, network access, `$CARGO_HOME`, registry cache fallback, or lockfile mutation for the native claim.

#### Scenario: Supported topology binds selected artifacts

- GIVEN native planning for `native_rust_workspace_member_glob_planning` is ready
- AND the selected package or dependency edge participates in a supported topology rail
- WHEN Mantle executes the topology
- THEN producer artifacts MUST be built before consumers and bound by declared output digest evidence.
- AND the topology receipt MUST expose enough evidence to review the dependency source and artifact binding.

#### Scenario: Unsupported behavior blocks before rustc

- GIVEN a Rust workspace declares a shape for `native_rust_workspace_member_glob_planning` that is outside Mantle's bounded native fragment
- WHEN Mantle plans or executes the topology
- THEN Mantle MUST fail closed with a deterministic blocker identifying the unsupported class before invoking `rustc` for the affected claim.
- AND Mantle MUST NOT silently fall back to Cargo resolver behavior or ambient registry/cache material.

#### Scenario: CLI coverage proves positive and negative behavior

- GIVEN the implementation claims support for `native_rust_workspace_member_glob_planning`
- WHEN the relevant `rust_plan` or `rust_plan_cli` tests run
- THEN they MUST include at least one supported fixture and one unsupported fixture with deterministic assertions.

# Rust package planning spec delta

## ADDED Requirements

### r[rust_package_planning.native_rust_dev_dependency_test_topology_planning]

Mantle MUST represent `native_rust_dev_dependency_test_topology_planning` as bounded native Rust planning evidence and MUST emit deterministic blockers before execution when the surface would require unsupported Cargo behavior.

#### Scenario: Supported bounded surface is planned from native facts

- GIVEN a Rust workspace declares the bounded surface for `native_rust_dev_dependency_test_topology_planning`
- AND all required local or vendored-registry source facts are ready
- WHEN Mantle runs `rust-plan` for the workspace
- THEN Mantle MUST emit native receipt evidence identifying the selected package/member/dependency facts and source material.
- AND Mantle MUST NOT require Cargo orchestration, network access, `$CARGO_HOME`, registry cache fallback, or lockfile mutation for the native claim.

#### Scenario: Supported topology binds selected artifacts

- GIVEN native planning for `native_rust_dev_dependency_test_topology_planning` is ready
- AND the selected package or dependency edge participates in a supported topology rail
- WHEN Mantle executes the topology
- THEN producer artifacts MUST be built before consumers and bound by declared output digest evidence.
- AND the topology receipt MUST expose enough evidence to review the dependency source and artifact binding.

#### Scenario: Unsupported behavior blocks before rustc

- GIVEN a Rust workspace declares a shape for `native_rust_dev_dependency_test_topology_planning` that is outside Mantle's bounded native fragment
- WHEN Mantle plans or executes the topology
- THEN Mantle MUST fail closed with a deterministic blocker identifying the unsupported class before invoking `rustc` for the affected claim.
- AND Mantle MUST NOT silently fall back to Cargo resolver behavior or ambient registry/cache material.

#### Scenario: CLI coverage proves positive and negative behavior

- GIVEN the implementation claims support for `native_rust_dev_dependency_test_topology_planning`
- WHEN the relevant `rust_plan` or `rust_plan_cli` tests run
- THEN they MUST include at least one supported fixture and one unsupported fixture with deterministic assertions.

# Rust package planning spec delta

## ADDED Requirements

### r[rust_package_planning.native_rust_dev_dependency_test_topology_execution]

Mantle MUST execute bounded dev-dependency test topologies from explicit native planning and execution receipt material, and MUST fail closed before `rustc` when execution would require unsupported Cargo test behavior.

#### Scenario: Supported dev-dependency test topology executes from native facts

- GIVEN native planning for `native_rust_dev_dependency_test_topology_planning` is ready
- AND the workspace declares a bounded test target that consumes a vendored-registry dev-dependency with ready source facts
- WHEN Mantle executes the dev-dependency test topology
- THEN Mantle MUST execute required producer units before the test consumer using explicit derivation args and env.
- AND Mantle MUST bind produced artifacts into declared dependency/input/`--extern` surfaces by digest evidence.
- AND Mantle MUST NOT invoke Cargo as planner, test runner, executor, cache provider, or topology repair mechanism.

#### Scenario: Execution receipt preserves bounded test evidence

- GIVEN a supported dev-dependency test topology executes
- WHEN Mantle records the execution receipt
- THEN the receipt MUST identify the package/member, test target, selected dev-dependency, source closure digest, ordered unit execution receipts, artifact digests, toolchain identity, rustc argument digest, bounded claim, and stable receipt hash.
- AND the receipt MUST NOT claim full Cargo test compatibility, doctest/run/bench support, hidden harness behavior, or feature resolver parity beyond the bounded fixture.

#### Scenario: Unsupported test surfaces block before rustc

- GIVEN a dev-dependency test topology requires unsupported Cargo behavior such as doctest/run/bench modes, hidden harness discovery, missing/stale vendor material, missing test-unit derivations, or unsupported resolver behavior
- WHEN Mantle evaluates the topology for execution
- THEN Mantle MUST emit a deterministic blocker before invoking `rustc` for the affected claim.
- AND Mantle MUST NOT fall back to Cargo, `$CARGO_HOME`, registry caches, target directories, or network access.

#### Scenario: Normal build topology does not consume dev-dependency material

- GIVEN a package has dev-dependency test topology evidence
- WHEN Mantle executes normal build topology rather than the bounded test topology
- THEN Mantle MUST NOT silently include dev-dependency artifacts in the normal build claim.
- AND any normal-build shape that would require dev-dependency material MUST emit deterministic blocker evidence.

#### Scenario: CLI coverage proves positive and negative behavior

- GIVEN the implementation claims support for `native_rust_dev_dependency_test_topology_execution`
- WHEN the relevant `rust_plan` or `rust_plan_cli` tests run
- THEN they MUST include at least one supported execution fixture and one unsupported fixture with deterministic assertions.

### r[rust_package_planning.native_registry_workspace_dependency_topology_execution]

Mantle MUST execute bounded workspace-inherited vendored-registry dependency topologies from explicit native planning and execution receipt material, and MUST fail closed before `rustc` when execution would require unsupported Cargo resolver, cache, or network behavior.

#### Scenario: Supported workspace-inherited registry topology executes from native facts

- GIVEN native planning for `native_registry_workspace_dependency_topology_planning` is ready
- AND a workspace root declares a supported vendored-registry dependency in `[workspace.dependencies]`
- AND a workspace member declares that dependency with `{ workspace = true }`
- WHEN Mantle executes the workspace-dependency topology
- THEN Mantle MUST execute inherited registry producer units before the workspace member consumer using explicit derivation args and env.
- AND Mantle MUST bind produced artifacts into declared dependency/input/`--extern` surfaces by digest evidence.
- AND Mantle MUST NOT invoke Cargo as planner, resolver, executor, cache provider, or topology repair mechanism.

#### Scenario: Execution receipt preserves bounded workspace-dependency evidence

- GIVEN a supported workspace-dependency topology executes
- WHEN Mantle records the execution receipt
- THEN the receipt MUST identify the workspace root, member package, inherited dependency package ids, ordered unit execution receipts, artifact digests, toolchain identity, rustc argument digests, bounded claim, blocker when present, and stable receipt hash.
- AND the receipt MUST NOT claim full Cargo resolver compatibility, version solving, network/index access, ambient registry cache fallback, or generalized workspace scheduling.

#### Scenario: Unsupported or stale workspace-dependency surfaces block before rustc

- GIVEN a workspace-dependency topology requires unsupported member-side features/default-feature/platform behavior, missing/stale vendor material, missing source facts, missing unit derivations, or resolver behavior outside the bounded fragment
- WHEN Mantle evaluates the topology for execution
- THEN Mantle MUST emit a deterministic blocker before invoking `rustc` for the affected claim.
- AND Mantle MUST NOT fall back to Cargo, `$CARGO_HOME`, registry caches, target directories, or network access.

#### Scenario: Explicit CLI receipt is separate from general topology execution

- GIVEN a package has workspace-dependency topology evidence
- WHEN Mantle executes normal `--execute-topology`
- THEN Mantle MUST NOT silently emit the dedicated `native_registry_workspace_dependency_topology_execution` receipt.
- AND the dedicated receipt MUST only be emitted by the explicit workspace-dependency topology execution path.

#### Scenario: CLI coverage proves positive and negative behavior

- GIVEN the implementation claims support for `native_registry_workspace_dependency_topology_execution`
- WHEN the relevant `rust_plan_cli` tests run
- THEN they MUST include at least one supported execution fixture and one unsupported fixture with deterministic assertions.

### r[rust_package_planning.native_registry_patch_source_topology_execution]

Mantle MUST execute bounded local patch-source registry topologies from explicit native planning and execution receipt material, and MUST fail closed before `rustc` when execution would require unsupported Cargo resolver, cache, registry, or network behavior.

#### Scenario: Supported local patch-source topology executes from native facts

- GIVEN native planning for `native_registry_patch_source_planning` is ready
- AND a package declares a supported local `[patch.crates-io]` replacement for a dependency
- WHEN Mantle executes the patch-source topology
- THEN Mantle MUST execute the patched local source producer before the consumer using explicit derivation args and env.
- AND Mantle MUST bind produced artifacts into declared dependency/input/`--extern` surfaces by digest evidence.
- AND Mantle MUST NOT invoke Cargo as planner, resolver, executor, cache provider, or topology repair mechanism.

#### Scenario: Execution receipt preserves bounded patch-source evidence

- GIVEN a supported patch-source topology executes
- WHEN Mantle records the execution receipt
- THEN the receipt MUST identify the consumer package, patch source package ids, ordered unit execution receipts, artifact digests, toolchain identity, rustc argument digests, bounded claim, blocker when present, and stable receipt hash.
- AND the receipt MUST NOT claim full Cargo resolver compatibility, version solving, non-crates.io patch registry support, git patch support, network/index access, ambient registry cache fallback, or generalized patch scheduling.

#### Scenario: Unsupported or stale patch-source surfaces block before rustc

- GIVEN a patch-source topology requires unsupported patch registry/source behavior, missing/stale patch material, missing source facts, missing unit derivations, or resolver behavior outside the bounded fragment
- WHEN Mantle evaluates the topology for execution
- THEN Mantle MUST emit a deterministic blocker before invoking `rustc` for the affected claim.
- AND Mantle MUST NOT fall back to Cargo, `$CARGO_HOME`, registry caches, target directories, or network access.

#### Scenario: Explicit CLI receipt is separate from general topology execution

- GIVEN a package has patch-source topology evidence
- WHEN Mantle executes normal `--execute-topology`
- THEN Mantle MUST NOT silently emit the dedicated `native_registry_patch_source_topology_execution` receipt.
- AND the dedicated receipt MUST only be emitted by the explicit patch-source topology execution path.

#### Scenario: CLI coverage proves positive and negative behavior

- GIVEN the implementation claims support for `native_registry_patch_source_topology_execution`
- WHEN the relevant `rust_plan_cli` tests run
- THEN they MUST include at least one supported execution fixture and one unsupported fixture with deterministic assertions.

### r[rust_package_planning.native_workspace_package_inheritance_planning]



Mantle MUST model bounded Cargo workspace package-version inheritance in native package/target planning, and MUST fail closed when inherited package identity cannot be resolved from explicit workspace manifest facts.

#### Scenario: Native package planning resolves workspace package version inheritance

- GIVEN a workspace root declares `[workspace.package].version`
- AND a package manifest declares `version.workspace = true`
- WHEN Mantle computes native package/target planning facts
- THEN Mantle MUST resolve the package version from the workspace root manifest.
- AND Mantle MUST use the resolved version for native package identity, oracle comparison, target facts, and downstream unit graph material.

#### Scenario: Unsupported workspace package inheritance blocks deterministically

- GIVEN a package manifest requests unsupported package version inheritance material or the workspace root lacks `[workspace.package].version`
- WHEN Mantle computes native package/target planning facts
- THEN Mantle MUST emit a deterministic native package planning blocker.
- AND Mantle MUST NOT fall back to Cargo-derived package identity as the native fact.

#### Scenario: CLI coverage proves inherited version facts

- GIVEN the implementation claims support for `native_workspace_package_inheritance_planning`
- WHEN the relevant `rust_plan_cli` tests run
- THEN at least one fixture MUST prove `version.workspace = true` is resolved into native package facts.

# Rust package planning spec delta

## ADDED Requirements

### r[rust_package_planning.native_vendor_deps_source_layout_planning]

Mantle MUST bind checkout-local `vendor-deps/` registry source layouts as explicit native planning material.

#### Scenario: conventional vendor-deps source root is discovered

- GIVEN a Rust workspace checkout contains `vendor-deps/`
- WHEN native registry source planning runs
- THEN the planner MUST treat that directory as declared local registry source material
- AND it MUST NOT require `$CARGO_HOME`, network access, or an ambient Cargo cache for those sources.

#### Scenario: unversioned vendor directory binds registry package identity

- GIVEN `Cargo.lock` contains a registry package with checksum evidence
- AND the checkout contains `vendor-deps/<crate>/Cargo.toml`
- WHEN native registry source planning binds the package
- THEN it MUST use the local manifest and `.cargo-checksum.json`
- AND it MUST emit source digest evidence for the local tree.

#### Scenario: Cargo oracle comparison uses package identity

- GIVEN Cargo metadata reports the same registry package from a Cargo cache manifest path
- AND native planning bound the package from `vendor-deps/<crate>/Cargo.toml`
- WHEN native package/target comparison runs
- THEN it MUST match by package ID before comparing manifest paths
- AND it MUST NOT emit `cargo-oracle-missing-package` for that package solely because the local vendor path differs from Cargo's cache path.

# Rust package planning spec delta

## ADDED Requirements

### r[rust_package_planning.native_rlib_crate_type_derivation_planning]

Mantle MUST treat Cargo target-kind arrays containing `rlib` as library-compatible for native Rust unit derivation planning.

#### Scenario: rlib target kind is planned as a library

- GIVEN a Cargo unit target kind array contains `rlib`
- WHEN unit derivation planning classifies the unit
- THEN it MUST select `target_kind = "lib"`
- AND it MUST NOT emit `unsupported-target-kind` solely because the kind array lacks the broader `lib` marker.

#### Scenario: mixed cdylib and rlib target keeps bounded claim

- GIVEN a Cargo unit target kind array contains both `cdylib` and `rlib`
- WHEN Mantle plans reviewable Rust unit derivations
- THEN it MUST plan the unit through the rlib/library facet
- AND it MUST NOT claim cdylib output production.

#### Scenario: non-rlib unsupported target remains fail-closed

- GIVEN a target kind array contains no supported `lib`, `rlib`, `bin`, `custom-build`, `proc-macro`, or bounded test marker
- WHEN unit derivation planning classifies the unit
- THEN it MUST emit a deterministic unsupported-target blocker before execution.

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

# Rust Package Planning Specification Delta

## ADDED Requirements

### Requirement: Native self target-cfg predicate scope

r[rust_package_planning.native_self_target_cfg_predicate_scope] Mantle MUST evaluate bounded build-relevant target dependency cfg predicates from native manifest facts before normal build topology execution, without consulting Cargo as an executor or resolver fallback.

#### Scenario: Supported nested target-cfg predicates are deterministic

GIVEN a native package manifest with target dependency tables
WHEN the tables use supported atoms (`unix`, `windows`, exact target triple names, `target_os`, `target_arch`, `target_family`, `target_vendor`, `target_env`, `target_abi`, `target_endian`, `target_pointer_width`, `target_has_atomic`) and supported composition operators (`not`, `any`, `all`)
THEN Mantle MUST decide each target dependency table as selected or not-selected from the explicit active target triple
AND selected dependencies MUST enter native dependency facts only through existing path or declared-registry source bindings.

#### Scenario: Unsupported or malformed cfg syntax remains fail-closed

GIVEN a native package manifest with a target dependency table
WHEN the table uses cfg syntax outside the bounded predicate fragment
THEN Mantle MUST emit an `unsupported-target-cfg-surface` blocker before rustc execution
AND MUST NOT silently ask Cargo to decide the dependency surface.


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

### Requirement: Native git dependency source scope

r[rust_package_planning.native_git_dependency_source_scope] Mantle MUST represent supported locked git dependency sources as explicit native source facts before those packages participate in Cargo-free native package facts, unit graphs, or topology execution.

#### Scenario: Locked git dependency source facts are derived from captured source closure

GIVEN a Cargo lockfile contains a git package entry with a URL and resolved revision
AND the captured Rust source closure contains the same package identity with a readable manifest path and source root
WHEN Mantle computes native Rust source-planning facts for the current invocation
THEN Mantle MUST record a native git source fact containing package name, version, package identity, git URL, resolved revision, lockfile digest, manifest path, source root, and BLAKE3 source-tree digest
AND the source fact MUST treat the git URL and revision as identity evidence only, not as a hard-coded pull mechanism.
AND the source fact MUST identify that source bytes are bounded to provider-agnostic captured source-closure material, with BLAKE3 content evidence suitable for the default snix-store-backed source path, not a network fetch or version-solving result.

#### Scenario: Git dependency edges resolve only through ready git source facts

GIVEN a native package has a selected dependency whose Cargo lockfile source is git-backed
AND native git source planning has a ready fact for that dependency package identity
WHEN Mantle computes native package dependency facts
THEN Mantle MUST resolve the dependency edge to the git source fact only when package name, optional manifest version, git URL, and requested rev/tag/branch identity match the captured fact instead of emitting `unsupported-non-path-dependency`
AND the resolved dependency MUST be eligible for downstream native unit graph and topology planning under the same explicit-source rules as supported registry/path dependencies.

#### Scenario: Git source scope fails closed for undeclared or ambiguous material

GIVEN a selected git dependency lacks captured source-closure material, has an unreadable manifest path, has no resolved revision, has mismatched requested rev/tag/branch identity, has multiple ambiguous package roots for one package identity, or would require network or `$CARGO_HOME` discovery outside the captured source closure
WHEN Mantle computes native source or package dependency facts
THEN Mantle MUST emit a deterministic native git source blocker before rustc execution
AND Mantle MUST NOT claim native package-target, unit graph, host-unit graph, or topology execution readiness for packages depending on that unresolved git source.

#### Scenario: Git source facts are receipt-bound and auditable

GIVEN Mantle emits `rust-plan` JSON evidence for a workspace with supported locked git dependency sources
WHEN native git source planning finishes
THEN the receipt MUST include ready status, ordered git source facts, lockfile/source digests, oracle/source-closure comparison evidence, blockers when present, and a stable receipt hash
AND the receipt MUST keep the bounded claim explicit: locked git packages with captured local source-closure material only, with no general Cargo git compatibility claim.

#### Scenario: Native git source scope covers the self-probe blocker

GIVEN Mantle's self `rust-plan --execute-topology` probe currently reports `snix-castore` missing native package facts because dependency `wu-manber` is outside the bounded path-or-declared-registry fragment
WHEN this change is implemented for locked git source facts
THEN focused verification MUST show that `wu-manber` no longer produces `unsupported-non-path-dependency` solely because it is git-backed
AND remaining blockers, if any, MUST be recorded as new deterministic classes with baseline/current evidence.

### Requirement: Native registry transitive producer coverage

r[rust_package_planning.native_registry_transitive_producer_coverage] Mantle MUST provide executable native producer coverage for supported transitive registry dependency artifacts before claiming unit-derivation graph or topology readiness.

#### Scenario: Supported transitive registry dependency gets a producer unit

GIVEN native package-target planning has ready facts for a registry-backed dependency package with ready native source facts and a supported `lib` target
AND a supported native target unit consumes a dependency artifact for that registry package
WHEN Mantle computes native unit graph and unit-derivation graph evidence
THEN Mantle MUST include an eligible producer `lib` unit for that registry package or otherwise keep graph readiness false with a deterministic blocker.
AND the producer unit MUST bind the package identity, source digest, rustc argument digest, declared output, and dependency artifacts using explicit native facts.

#### Scenario: Missing producer coverage fails before topology execution

GIVEN a supported consumer unit has a dependency artifact whose package lacks native package facts, ready source facts, a supported `lib` target, or a selected build-mode producer unit
WHEN Mantle computes native unit graph or unit-derivation graph evidence
THEN Mantle MUST emit a deterministic planning blocker naming the consumer package, dependency package, and missing producer reason.
AND Mantle MUST NOT report `unit_derivation_graph.ready=true`, native unified topology readiness, or topology execution readiness for that graph.

#### Scenario: Transitive producer execution remains source-closure bounded

GIVEN `rust-plan --execute-topology` executes a graph containing a transitive registry-backed producer
WHEN Mantle executes the producer and downstream consumers
THEN Mantle MUST execute the producer before affected consumers using only explicit derivation args, source facts, BLAKE3 source digests, dependency artifacts, host artifacts, and declared outputs.
AND Mantle MUST NOT use Cargo as the build orchestrator or read undeclared registry caches, git checkouts, target directories, or network locations to materialize the producer.

#### Scenario: Self-probe blocker moves past itertools producer coverage

GIVEN the current pushed-head self probe reports `missing-dependency-producer` for `registry+https://github.com/rust-lang/crates.io-index#itertools@0.10.5`
WHEN native registry transitive producer coverage is implemented
THEN focused verification MUST show that `itertools@0.10.5` no longer fails solely because no producer unit exists.
AND remaining blockers, if any, MUST be recorded with deterministic classes and baseline/current evidence.

### Requirement: Native host dependency producer coverage

r[rust_package_planning.native_host_dependency_producer_coverage] Mantle MUST provide bounded producer coverage for selected host-unit dependency artifacts before claiming unified topology execution readiness.

#### Scenario: Proc-macro host dependency producer executes before host consumer

GIVEN a selected native host unit consumes a dependency artifact whose package has a supported native proc-macro host producer
AND the dependency package has ready source, package, host-unit, and derivation graph facts
WHEN Mantle executes unified topology evidence
THEN Mantle MUST execute the proc-macro host producer before the consuming host unit.
AND Mantle MUST bind the produced proc-macro host artifact path into the consuming host unit's dependency artifact surface before invoking `rustc`.

#### Scenario: Target library host dependency producer remains supported

GIVEN a selected native host unit consumes a dependency artifact whose package has a supported native target `lib` producer
WHEN Mantle computes unified topology ordering
THEN Mantle MUST execute the target `lib` producer and its target dependencies before the consuming host unit.
AND Mantle MUST bind the produced target library artifact path into the consuming host unit's dependency artifact surface before invoking `rustc`.

#### Scenario: Missing host dependency producer fails closed

GIVEN a selected native host unit consumes a dependency artifact whose package has neither a supported target `lib` producer nor a supported proc-macro host producer
WHEN Mantle computes unified topology ordering
THEN Mantle MUST return a deterministic `missing-host-dependency-producer` blocker naming the dependency package.
AND Mantle MUST NOT invoke the host consumer using ambient Cargo state, registry caches, target directories, or network locations.

#### Scenario: Self-probe moves past rustversion host dependency producer coverage

GIVEN the current self probe reports `missing-host-dependency-producer` for `registry+https://github.com/rust-lang/crates.io-index#rustversion@1.0.22`
WHEN native host dependency producer coverage is implemented
THEN focused verification MUST show that `rustversion@1.0.22` no longer fails solely because its proc-macro host producer is ignored.
AND remaining blockers, if any, MUST be recorded with deterministic classes and baseline/current evidence.

### Requirement: Rust topology tool PATH environment

r[rust_package_planning.rust_topology_tool_path_env] Mantle MUST provide a bounded tool PATH to Rust topology child processes when the caller supplies one.

#### Scenario: Rustc child keeps invocation tool PATH

GIVEN the caller invokes Rust topology execution with a non-empty `PATH`
WHEN Mantle invokes a selected `rustc` unit after clearing ambient environment
THEN Mantle MUST pass that `PATH` to the `rustc` child process.
AND Mantle MUST still pass only explicit derivation environment keys plus the tool PATH.

#### Scenario: Build-script metadata child keeps invocation tool PATH

GIVEN a selected custom-build unit is rebuilt and then executed for metadata
WHEN Mantle invokes the produced build-script executable after clearing ambient environment
THEN Mantle MUST pass the same non-empty tool `PATH` to the build-script child process.
AND Mantle MUST still set required metadata variables such as `OUT_DIR` explicitly.

#### Scenario: Empty tool PATH stays absent

GIVEN the caller has no usable `PATH`
WHEN Mantle constructs a Rust topology child environment
THEN Mantle MUST omit `PATH` instead of fabricating host-specific defaults.
AND downstream tool failures MUST remain deterministic execution blockers.

### Requirement: Rust topology external linker mode

r[rust_package_planning.rust_topology_external_linker] Mantle MUST avoid host-rustup self-contained linker wrapper failures when selected Rust unit args do not explicitly request them.

#### Scenario: Runtime args disable self-contained linker by default

GIVEN a selected Rust unit has no `link-self-contained` rustc codegen option
WHEN Mantle executes that unit through the Rust topology rail
THEN Mantle MUST add `-C link-self-contained=no` to the runtime rustc invocation.
AND Mantle MUST keep the selected unit's original reviewable args otherwise unchanged.

#### Scenario: Explicit self-contained linker choice is preserved

GIVEN a selected Rust unit already contains a `link-self-contained` rustc codegen option
WHEN Mantle derives runtime rustc args
THEN Mantle MUST NOT add another `link-self-contained` option.
AND Mantle MUST preserve the selected unit's explicit value.

#### Scenario: Stale rustup linker wrapper blocker moves

GIVEN topology execution currently fails because rustup's `gcc-ld/ld.lld` wrapper references a missing Nix store `ld-wrapper.sh`
WHEN external linker mode is applied
THEN self-probe verification MUST show that this specific missing `ld-wrapper.sh` failure no longer blocks the first topology unit.
AND any remaining blocker MUST be recorded with deterministic class and evidence.

### Requirement: Native Rust manifest edition derivations

r[rust_package_planning.native_manifest_edition_derivations] Mantle MUST propagate each native package manifest edition into every generated native Rust unit derivation that compiles that package target.

#### Scenario: Declared edition is used for native derivation args

GIVEN a supported native package manifest declares `edition = "2024"`
WHEN Mantle plans native target or host unit derivations for that package
THEN Mantle MUST emit rustc args with `--edition 2024` for that package's generated unit derivations.
AND Mantle MUST NOT replace the declared edition with a hard-coded default.

#### Scenario: Workspace-inherited edition is used

GIVEN a supported native package manifest declares `edition.workspace = true`
AND the root workspace manifest declares `workspace.package.edition = "2024"`
WHEN Mantle plans native target or host unit derivations for that package
THEN Mantle MUST emit rustc args with `--edition 2024`.
AND Mantle MUST keep the inherited edition deterministic in native package/target facts.

#### Scenario: Missing edition uses Cargo default

GIVEN a supported native package manifest omits `package.edition`
WHEN Mantle plans native target or host unit derivations for that package
THEN Mantle MUST emit rustc args with Cargo's default `--edition 2015`.
AND Mantle MUST keep the default deterministic in native package/target facts.

#### Scenario: Rust 2024 topology blocker moves

GIVEN topology execution currently fails because a native proc-macro package that declares Rust 2024 is invoked with `--edition 2021`
WHEN native edition propagation is applied
THEN self-probe verification MUST show that the Rust 2024 let-chain edition error no longer blocks the first topology units.
AND any remaining blocker MUST be recorded with deterministic class and evidence.

### Requirement: Native build-script execution environment

r[rust_package_planning.native_build_script_execution_env] Mantle MUST execute native Rust build scripts with a bounded deterministic Cargo-like environment derived from the selected unit and invocation.

#### Scenario: Build script receives deterministic tool and target environment

GIVEN a native custom-build unit is executed for metadata
WHEN Mantle launches the compiled build-script executable
THEN Mantle MUST set `RUSTC` to the selected rustc path.
AND Mantle MUST set `HOST`, `TARGET`, and `PROFILE` deterministically from the active Rust topology invocation.
AND Mantle MUST preserve the cleared-env boundary except for the bounded PATH and explicit build-script variables.

#### Scenario: Build script runs from package root

GIVEN a native custom-build unit source path has a parent package directory
WHEN Mantle launches the compiled build-script executable
THEN Mantle MUST set the child current directory to that package directory.
AND Mantle MUST set `CARGO_MANIFEST_DIR` to the same package directory.

#### Scenario: Missing RUSTC topology blocker moves

GIVEN topology execution currently fails because a build script reports `Environment variable $RUSTC is not set during execution of build script`
WHEN build-script env binding is applied
THEN self-probe verification MUST show that this missing `$RUSTC` failure no longer blocks the first topology units.
AND any remaining blocker MUST be recorded with deterministic class and evidence.

### Requirement: Native build-script package name environment

r[rust_package_planning.native_build_script_package_name_env] Mantle MUST set native build-script `CARGO_PKG_NAME` from the manifest package name rather than the build-script target name.

#### Scenario: Build script receives manifest package name

GIVEN a native package manifest name contains a hyphen
AND Mantle executes that package's custom-build unit
WHEN Mantle derives the build-script child environment
THEN Mantle MUST set `CARGO_PKG_NAME` to the manifest package name with hyphen spelling preserved.
AND Mantle MUST NOT derive `CARGO_PKG_NAME` from `build-script-build` or other target names.

#### Scenario: Legacy unit fallback remains deterministic

GIVEN a build-script unit lacks explicit package-name env data
WHEN Mantle derives the build-script child environment
THEN Mantle MAY fall back to the deterministic crate-style target name.
AND Mantle MUST keep that fallback bounded to units without package-derived name data.

#### Scenario: Package-name fix preserves current topology frontier

GIVEN topology execution currently advances through successful build-script metadata runs after build-script env binding
WHEN package-name env binding is applied
THEN self-probe verification MUST show that the topology frontier does not regress to the old missing `$RUSTC` blocker.
AND any remaining blocker MUST be recorded with deterministic class and evidence.

### Requirement: Native proc-macro host dependency binding

r[rust_package_planning.native_proc_macro_host_dependency_binding] Mantle MUST bind native proc-macro host units with Cargo-equivalent compiler and dependency inputs before rustc execution.

#### Scenario: Proc-macro host unit receives compiler proc_macro extern

GIVEN a native package declares `[lib] proc-macro = true`
WHEN Mantle derives the proc-macro host rustc invocation
THEN Mantle MUST include the compiler-provided `proc_macro` extern input.
AND Mantle MUST NOT require a produced package artifact for `proc_macro`.

#### Scenario: Proc-macro host unit receives selected normal dependencies

GIVEN a native proc-macro package has selected normal dependencies such as `proc-macro2`, `quote`, or `syn`
WHEN Mantle plans native host-unit dependency artifacts
THEN Mantle MUST include Cargo-selected normal dependencies as host-unit dependency artifacts.
AND Mantle MUST NOT include unselected manifest-only dependency edges.
AND Mantle MUST bind selected artifacts to produced target library paths before executing the proc-macro host unit.

#### Scenario: Proc-macro dependency features reach rustc cfgs

GIVEN a native proc-macro dependency crate has selected features such as `syn/parsing` or `syn/visit-mut`
WHEN Mantle derives that dependency crate's native rustc invocation
THEN Mantle MUST pass the selected features as deterministic `--cfg feature=...` rustc args.
AND Mantle MUST preserve the no-feature default when no features are selected.

#### Scenario: Proc-macro dependencies run before proc-macro host unit

GIVEN a combined native topology contains a proc-macro host unit and its normal library dependency producers
WHEN Mantle orders the topology
THEN Mantle MUST schedule dependency library producers before the proc-macro host unit.
AND Mantle MUST preserve deterministic blocker reporting if a dependency producer is missing.

#### Scenario: Proc-macro frontier moves

GIVEN topology execution currently blocks while compiling `async-stream-impl` with unresolved `proc_macro` and dependency imports
WHEN proc-macro host dependency binding is applied
THEN self-probe verification MUST show that this unresolved-import blocker no longer stops the topology at `async-stream-impl`.
AND any remaining blocker MUST be recorded with deterministic class and evidence.

### Requirement: Native linked build-script metadata environment

r[rust_package_planning.native_linked_build_script_metadata_env] Mantle MUST propagate bounded linked dependency build-script metadata into dependent build-script environments using Cargo-compatible `DEP_*` variables.

#### Scenario: Package-level build path is planned

GIVEN a native package manifest declares `[package] build = "builder/main.rs"`
WHEN Mantle plans native package targets
THEN Mantle MUST include a custom-build target for that package-level build script.
AND Mantle MUST use the deterministic Cargo-compatible custom-build target name `build-script-build`.

#### Scenario: Build-script metadata captures safe custom keys

GIVEN a build script emits safe custom metadata lines such as `cargo:include=/path/include`
WHEN Mantle parses build-script stdout
THEN Mantle MUST record the metadata key and value deterministically.
AND Mantle MUST continue to reject malformed rustc metadata before execution proceeds.

#### Scenario: Linked dependency metadata reaches dependent build script

GIVEN a package has a normal dependency whose manifest declares `links = "aws_lc_0_39_1"`
AND that linked dependency build script emits `cargo:include=/path/include`
WHEN Mantle runs the dependent package's build script
THEN Mantle MUST set `DEP_AWS_LC_0_39_1_INCLUDE=/path/include` in the child environment.
AND Mantle MUST keep the environment bounded to derived `DEP_*` variables plus the existing build-script variables.

#### Scenario: Linked dependency build script runs before dependent build script

GIVEN a dependent build script needs linked dependency metadata
WHEN Mantle orders the combined native topology
THEN Mantle MUST schedule the linked dependency's custom-build host unit before the dependent build script.
AND Mantle MUST report a deterministic blocker if no linked dependency metadata producer exists.

#### Scenario: Metadata frontier moves

GIVEN topology execution currently blocks in `aws-lc-rs` with `missing DEP_AWS_LC_ include`
WHEN linked build-script metadata propagation is applied
THEN self-probe verification MUST show that this missing `DEP_AWS_LC_` blocker no longer stops `aws-lc-rs` build-script execution.
AND any remaining blocker MUST be recorded with deterministic class and evidence.

### Requirement: Native build-script package metadata environment

r[rust_package_planning.native_build_script_package_metadata_env] Mantle MUST provide bounded Cargo package metadata environment variables when compiling and running native Rust units.

#### Scenario: Build-script compile-time package version is available

GIVEN a native package manifest resolves to package version `0.39.1`
WHEN Mantle compiles that package's custom-build host unit with direct rustc
THEN Mantle MUST set `CARGO_PKG_VERSION=0.39.1` in the rustc environment.
AND Mantle MUST set deterministic version component env vars for major, minor, patch, and pre-release.

#### Scenario: Optional package metadata defaults are deterministic

GIVEN a native package manifest omits optional package metadata such as description, homepage, license, license-file, repository, readme, rust-version, or authors
WHEN Mantle derives package metadata environment variables
THEN Mantle MUST set those bounded `CARGO_PKG_*` variables to empty strings rather than reading ambient process state.

#### Scenario: Build-script runtime package metadata matches compile-time data

GIVEN Mantle executes a compiled custom-build host unit
WHEN Mantle derives the child build-script environment
THEN Mantle MUST pass through the bounded `CARGO_PKG_*` package metadata variables from the derivation environment.
AND Mantle MUST continue to preserve deterministic fallback behavior only for legacy units that lack package-derived name data.

#### Scenario: Package metadata frontier moves

GIVEN topology execution currently blocks while compiling `aws-lc-sys` with missing `CARGO_PKG_VERSION`
WHEN package metadata env binding is applied
THEN self-probe verification MUST show that this missing `CARGO_PKG_VERSION` blocker no longer stops `aws-lc-sys` build-script compilation.
AND any remaining blocker MUST be recorded with deterministic class and evidence.

### Requirement: Native build-script target cfg environment

r[rust_package_planning.native_build_script_target_cfg_env] Mantle MUST provide bounded Cargo-compatible target cfg environment variables when running native build scripts.

#### Scenario: Target architecture reaches build-script runtime

GIVEN Mantle runs a native custom-build host unit for active target `x86_64-unknown-linux-gnu`
WHEN Mantle derives the child build-script environment
THEN Mantle MUST set `CARGO_CFG_TARGET_ARCH=x86_64`.
AND Mantle MUST derive the value from the selected target triple rather than ambient process environment.

#### Scenario: Common target cfg fields are deterministic

GIVEN Mantle runs a native custom-build host unit for a supported target triple
WHEN Mantle derives target cfg environment variables
THEN Mantle MUST set deterministic values for target vendor, os, env, family, endian, and pointer width.
AND Mantle MUST use empty strings for known-empty Cargo fields such as target env on triples without an env component.

#### Scenario: Build-script target cfg frontier moves

GIVEN topology execution currently runs `aws-lc-sys` and blocks on missing `CARGO_CFG_TARGET_ARCH`
WHEN target cfg env binding is applied
THEN self-probe verification MUST show that this missing `CARGO_CFG_TARGET_ARCH` blocker no longer stops `aws-lc-sys` build-script execution.
AND any remaining blocker MUST be recorded with deterministic class and evidence.

### Requirement: Native build-script profile environment

r[rust_package_planning.native_build_script_profile_env] Mantle MUST provide bounded Cargo-compatible profile environment variables when running native build scripts.

#### Scenario: cc-rs profile opt level reaches build-script runtime

GIVEN Mantle runs a native custom-build host unit with the default dev profile
WHEN Mantle derives the child build-script environment
THEN Mantle MUST set `OPT_LEVEL=0`.
AND Mantle MUST derive the value from Mantle's selected profile rather than ambient process environment.

#### Scenario: Debug and job profile fields are deterministic

GIVEN Mantle runs a native custom-build host unit with a supported profile
WHEN Mantle derives profile environment variables
THEN Mantle MUST set deterministic `DEBUG` and `NUM_JOBS` values.
AND Mantle MUST use bounded profile defaults rather than reading ambient Cargo state.

#### Scenario: Profile env frontier moves

GIVEN topology execution currently runs `aws-lc-sys` and blocks inside cc-rs on missing `OPT_LEVEL`
WHEN profile env binding is applied
THEN self-probe verification MUST show that this missing `OPT_LEVEL` blocker no longer stops `aws-lc-sys` build-script execution.
AND any remaining blocker MUST be recorded with deterministic class and evidence.

### Requirement: Native target proc-macro host extern binding

r[rust_package_planning.native_target_proc_macro_host_extern_binding] Mantle MUST bind planned proc-macro host artifacts into target rustc extern arguments before invoking target rustc.

#### Scenario: Target receives proc-macro extern from consumed host artifact

GIVEN a native target unit consumes a proc-macro host artifact through `consumed_host_artifacts`
AND the target unit has no matching dependency artifact placeholder for that proc-macro package
WHEN Mantle prepares the target unit for rustc execution
THEN Mantle MUST add a deterministic `--extern <proc-macro-crate>=<produced-host-artifact>` argument.
AND Mantle MUST bind the produced host artifact path into the target host-artifact receipt material.

#### Scenario: Existing dependency placeholder is not duplicated

GIVEN a native target unit consumes a proc-macro host artifact
AND the target unit already has a matching dependency artifact placeholder
WHEN Mantle prepares the target unit for rustc execution
THEN Mantle MUST rewrite the placeholder to the produced host artifact path.
AND Mantle MUST NOT add a duplicate `--extern` argument for the same proc-macro crate.

#### Scenario: Transitive proc-macro metadata remains discoverable

GIVEN a native target unit depends on a library whose metadata references a proc-macro host artifact
WHEN Mantle prepares the dependent target unit for rustc execution
THEN Mantle MUST add deterministic dependency search paths for produced proc-macro host artifact directories.
AND Mantle MUST NOT add a direct proc-macro `--extern` argument unless the target consumes that host artifact directly.

#### Scenario: Missing host artifact remains fail-closed

GIVEN a native target unit consumes a proc-macro host artifact
WHEN the corresponding host producer has not produced an artifact path
THEN Mantle MUST fail before invoking target rustc with deterministic missing-host-artifact diagnostics.
AND Mantle MUST NOT search the sysroot, Cargo target directories, or ambient caches to repair the missing proc-macro material.

#### Scenario: Darling proc-macro frontier moves

GIVEN topology execution currently blocks while compiling `darling@0.20.11` because `extern crate darling_macro` resolves from the sysroot
WHEN native target proc-macro host extern binding is applied
THEN self-probe verification MUST show that this sysroot `darling_macro` blocker no longer stops the topology at `darling@0.20.11`.
AND any remaining blocker MUST be recorded with deterministic class and evidence.

### Requirement: Native dependency cap-lints parity

r[rust_package_planning.native_dependency_cap_lints] Mantle MUST apply Cargo-compatible lint capping to native Rust units that come from non-local dependency sources.

#### Scenario: Registry dependency unit receives cap-lints allow

GIVEN a native Rust unit's source closure entry has kind `registry`
WHEN Mantle derives that unit's rustc arguments
THEN Mantle MUST include deterministic `--cap-lints allow` arguments.
AND Mantle MUST include those arguments for both target and host units.

#### Scenario: Git dependency unit receives cap-lints allow

GIVEN a native Rust unit's source closure entry has kind `git`
WHEN Mantle derives that unit's rustc arguments
THEN Mantle MUST include deterministic `--cap-lints allow` arguments.
AND Mantle MUST NOT require ambient Cargo state to make the decision.

#### Scenario: Local path unit remains uncapped

GIVEN a native Rust unit's source closure entry has kind `path`
WHEN Mantle derives that unit's rustc arguments
THEN Mantle MUST NOT add `--cap-lints allow` for that unit.
AND Mantle MUST keep local path crate lint behavior visible to the build.

#### Scenario: Derive-builder lint frontier moves

GIVEN topology execution currently blocks while compiling `derive_builder_core@0.20.2` because a dependency warning is treated as an error
WHEN native dependency cap-lints parity is applied
THEN self-probe verification MUST show that this lint-cap blocker no longer stops the topology at `derive_builder_core@0.20.2`.
AND any remaining blocker MUST be recorded with deterministic class and evidence.

### Requirement: Native link-lib build-script metadata

r[rust_package_planning.native_link_lib_metadata] Mantle MUST parse bounded Cargo-compatible `rustc-link-lib` build-script metadata without treating supported link-kind modifiers as malformed metadata.

#### Scenario: Modifier-bearing static link metadata is accepted

GIVEN a native build script emits `cargo:rustc-link-lib=static:+whole-archive=crypto_core`
WHEN Mantle parses build-script metadata
THEN Mantle MUST accept the directive as `rustc_link_lib` metadata.
AND Mantle MUST preserve the original directive value for deterministic rustc argument replay.

#### Scenario: Ordinary link metadata remains accepted

GIVEN a native build script emits `cargo:rustc-link-lib=static=aws_lc_0_39_1_crypto`
WHEN Mantle parses build-script metadata
THEN Mantle MUST accept the directive as `rustc_link_lib` metadata.
AND Mantle MUST continue to accept safe unqualified link names.

#### Scenario: Unsafe link metadata remains rejected

GIVEN a native build script emits empty, whitespace-containing, path-like, or unsupported-kind `rustc-link-lib` metadata
WHEN Mantle parses build-script metadata
THEN Mantle MUST reject the directive with deterministic `malformed-build-script-metadata` diagnostics.
AND Mantle MUST NOT pass unsafe link metadata through to rustc.

#### Scenario: Link-lib parser frontier moves

GIVEN topology execution currently blocks with `rustc-link-lib name must be a safe token`
WHEN bounded link-lib metadata parsing is applied
THEN self-probe verification MUST show that this parser blocker no longer stops the topology.
AND any remaining blocker MUST be recorded with deterministic class and evidence.

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

### Requirement: Native manifest links build-script env

r[rust_package_planning.native_manifest_links_env] Mantle MUST provide deterministic `CARGO_MANIFEST_LINKS` to native build scripts from the package manifest `links` field.

#### Scenario: Linked package receives links env

GIVEN a package manifest declares `[package].links = "ring_core_0_17_14"`
WHEN Mantle prepares a native build-script child environment
THEN the child environment MUST include `CARGO_MANIFEST_LINKS=ring_core_0_17_14`.

#### Scenario: Unlinked package receives empty links env

GIVEN a package manifest does not declare `[package].links`
WHEN Mantle prepares a native build-script child environment
THEN the child environment MUST include `CARGO_MANIFEST_LINKS` with an empty string value.

#### Scenario: Ring missing-links frontier moves

GIVEN topology execution currently reaches `ring`'s build script and panics while unwrapping `CARGO_MANIFEST_LINKS`
WHEN manifest links env parity is applied
THEN self-probe verification MUST show that the `CARGO_MANIFEST_LINKS` unwrap panic no longer stops topology.
AND any remaining blocker MUST be recorded with deterministic class and evidence.

### Requirement: Native registry dependency version resolution

r[rust_package_planning.native_registry_dependency_version_resolution] Mantle MUST prefer the vendored registry source identified by a dependency's declared version when same-name vendored registry packages exist.

#### Scenario: Exact version selects matching source

GIVEN vendored registry sources include `thiserror-impl` versions `1.0.69` and `2.0.18`
WHEN a dependency declares `thiserror-impl = "=2.0.18"`
THEN native package planning MUST select the `thiserror-impl@2.0.18` manifest path.

#### Scenario: Missing exact version fails closed

GIVEN vendored registry sources include multiple versions with the same package name
WHEN a dependency declares an exact version that is absent from vendored sources
THEN native package planning MUST report a deterministic missing-version blocker.

#### Scenario: Thiserror private API frontier moves

GIVEN topology execution currently links `thiserror@2.0.18` with the `thiserror-impl@1.0.69` proc macro
WHEN version-aware registry dependency resolution is applied
THEN self-probe verification MUST show `thiserror@2.0.18` consumes `thiserror-impl@2.0.18`.
AND any remaining blocker MUST be recorded with deterministic class and evidence.

### Requirement: Native proc-macro host selection normalizes Cargo target-name spelling

r[rust_package_planning.native_proc_macro_target_name_normalization] Native Rust host-unit planning MUST match Cargo-selected proc-macro host units by package id, proc-macro kind, and Rust crate-name spelling so manifest package-target names with hyphens match Cargo unit-graph target names with underscores.

#### Scenario: selected hyphenated proc macro is planned

- **GIVEN** a native package fact for `curve25519-dalek-derive` whose proc-macro target name uses hyphen spelling
- **AND** the Cargo unit graph contains the selected proc-macro host unit using `curve25519_dalek_derive` target-name spelling
- **WHEN** native host-unit planning matches selected host keys
- **THEN** the proc-macro host unit is planned
- **AND** the target consumer records that proc-macro as a consumed host artifact

#### Scenario: absent proc macro remains omitted

- **GIVEN** another proc-macro package exists in native package facts
- **AND** the Cargo unit graph does not contain that package's proc-macro host unit
- **WHEN** native host-unit planning matches selected host keys
- **THEN** the absent proc-macro package is not planned

### Requirement: Native custom-build host artifacts bind Cargo build-script aliases

r[rust_package_planning.native_custom_build_alias_binding] Native topology execution MUST bind same-package custom-build host artifacts to Cargo build-script dependency aliases `build_script_main` and `build_script_build` before resolving normal target dependency artifacts.

#### Scenario: build_script_main placeholder is satisfied by custom-build host artifact

GIVEN a target unit consumes its same-package custom-build host artifact
AND the target unit has a dependency placeholder named `build_script_main` for that same package
WHEN host artifacts are bound for execution
THEN the dependency placeholder MUST be rewritten to the produced custom-build executable path
AND execution MUST NOT wait for a same-package target artifact that cannot exist.

### Requirement: Native topology honors proc-macro manifest aliases and crate types

r[rust_package_planning.native_proc_macro_crate_type_classification] Native Rust topology planning MUST classify proc-macro libraries as host proc-macro units when native manifest facts or Cargo unit crate-type facts identify them as proc macros.

#### Scenario: normalized manifest proc_macro alias is planned as host proc macro

GIVEN a native package manifest declares `[lib] proc_macro = true`
WHEN Mantle plans native package targets
THEN the library target MUST be classified as a proc-macro target
AND later topology derivation MUST build it as a host proc-macro unit instead of a target lib.

#### Scenario: lib-shaped proc-macro crate type is planned as host proc macro

GIVEN a Cargo unit target has kind `lib` or `rlib`
AND its `crate_types` contains `proc-macro`
WHEN Mantle classifies the unit for derivation and host-artifact planning
THEN the unit MUST be planned as a proc-macro host unit
AND rustc arguments MUST include `--crate-type proc-macro`
AND the produced artifact MUST be available as a host proc-macro artifact to dependent target units.

#### Scenario: ordinary lib crate types remain target libs

GIVEN a Cargo unit target has kind `lib` or `rlib`
AND its `crate_types` does not contain `proc-macro`
WHEN Mantle classifies the unit for derivation and host-artifact planning
THEN the unit MUST remain a target lib unit
AND Mantle MUST NOT synthesize a proc-macro host artifact for it.

### Requirement: Native topology emits deterministic crate disambiguators

r[rust_package_planning.native_crate_disambiguators] Native Rust topology execution MUST pass deterministic rustc crate metadata disambiguators for every native rustc unit so same-name crate versions in one dependency graph do not collide.

#### Scenario: same crate name different package IDs

GIVEN two native units share the same Rust crate name
AND their package IDs or source digests differ
WHEN Mantle creates reviewable rustc derivations
THEN each derivation MUST include `-C metadata=<hash>`
AND the metadata values MUST differ.

#### Scenario: stable unit identity

GIVEN the same native unit identity and selected features
WHEN Mantle creates reviewable rustc derivations repeatedly
THEN the `-C metadata=<hash>` value MUST be stable
AND the value MUST be derived from reviewable unit facts, not from ambient Cargo target directories.

### Requirement: Native topology forwards bounded compile-time env

r[rust_package_planning.native_compile_env_allowlist] Native Rust topology execution MUST forward only explicitly allowlisted parent environment variables needed by direct rustc compile-time `env!(...)` uses.

#### Scenario: allowlisted sandbox shell reaches rustc

GIVEN the parent Mantle process has `SNIX_BUILD_SANDBOX_SHELL` set
WHEN Mantle constructs a native rustc child environment
THEN the child environment MUST include `SNIX_BUILD_SANDBOX_SHELL` with the parent value
AND explicit derivation env values MUST remain authoritative on key collision.

#### Scenario: unrelated ambient variables are rejected

GIVEN the parent Mantle process has an unrelated environment variable such as `LD_PRELOAD` or `SECRET_TOKEN`
WHEN Mantle constructs a native rustc child environment
THEN the child environment MUST NOT include that unrelated variable unless it appears in explicit derivation env.

### Requirement: Native self-package lib dependencies stay target-specific

r[rust_package_planning.native_self_package_lib_binding] Native Rust unit graph planning MUST bind same-package library artifacts only to package targets that actually consume that library, and MUST NOT attach package-level self dependency artifacts to the package library unit.

#### Scenario: package bin keeps same-package lib artifact

GIVEN a package has supported `lib` and `bin` targets
AND the Cargo-selected unit graph records the bin consuming the same-package lib artifact
WHEN Mantle plans native target units
THEN the native bin unit MUST contain exactly the same-package lib dependency artifact with the lib crate name.

#### Scenario: package lib rejects self dependency artifact

GIVEN a package has supported `lib` and `bin` targets
AND selected package dependency artifacts include the bin's same-package lib edge
WHEN Mantle plans the native lib unit
THEN the native lib unit MUST NOT contain a dependency artifact for its own package.

### Requirement: Native topology preserves transitive rustc dependency search paths

r[rust_package_planning.native_transitive_search_paths] Native Rust topology execution MUST preserve every produced target-library and proc-macro artifact directory as a rustc `-L dependency` search path until downstream units that may load transitive metadata have executed, and MUST NOT reduce that search set to one directory per package ID.

#### Scenario: duplicate package variants keep both search directories

GIVEN native topology execution has produced two library artifacts for the same package ID from distinct unit variants
WHEN a later Rust unit is prepared for direct rustc execution
THEN its rustc arguments MUST include `-L dependency` entries for both produced artifact parent directories.

#### Scenario: repeated search paths are deduplicated

GIVEN a Rust unit already contains a `-L dependency` entry for a produced artifact directory
WHEN native topology execution appends full produced search-path history
THEN the rustc arguments MUST NOT contain duplicate `-L dependency` pairs for that directory.

### Requirement: Native topology binds artifacts by unit variant identity

r[rust_package_planning.native_unit_variant_artifacts] Native Rust topology execution MUST bind dependency artifacts and rustc dependency search paths using the selected producer unit variant identity, not package ID alone.

#### Scenario: same-package variants do not overwrite direct binding

GIVEN native topology execution has produced two library artifacts for the same package ID from distinct unit variants
AND a later consumer dependency edge selects one of those producer variants
WHEN Mantle prepares the consumer rustc invocation
THEN the consumer `--extern` argument MUST point to the selected producer variant output
AND it MUST NOT be overwritten by another artifact with the same package ID.

#### Scenario: same-crate variants outside selected closure are excluded from rustc search

GIVEN two produced artifacts have the same Rust crate name and package ID but distinct unit variant identities
AND only one variant is in the consumer's selected dependency closure
WHEN Mantle prepares the consumer rustc invocation
THEN its `-L dependency` search paths MUST include the selected variant closure
AND MUST NOT include the unrelated same-crate variant path.

#### Scenario: ambiguous package-only producer fails before rustc

GIVEN a consumer dependency artifact lacks unit variant identity
AND multiple produced candidate artifacts share the same package ID and crate name
WHEN Mantle prepares the consumer rustc invocation
THEN Mantle MUST fail before invoking rustc with a deterministic ambiguous-producer blocker.

### Requirement: Native topology preserves real Cargo unit producer identity

r[rust_package_planning.native_real_unit_identity] Native Rust topology execution MUST preserve selected Cargo producer unit identity for dependency and host artifact binding.

#### Scenario: dependency edges use selected Cargo unit identity

GIVEN Cargo unit graph dependency material identifies a producer by unit index
WHEN Mantle lowers that dependency into native Rust dependency artifacts
THEN the dependency artifact MUST carry the selected producer unit ID derived from that exact Cargo unit entry
AND it MUST NOT substitute a package/crate/kind/mode pseudo-variant key.

#### Scenario: duplicate selected units remain distinct

GIVEN Cargo selects two supported target units for the same package ID, crate name, target kind, and mode
AND the units differ by selected Cargo identity or dependency facts
WHEN Mantle plans the native target unit graph
THEN both selected units MUST remain distinct native units
AND exact duplicate normalization MUST NOT collapse different unit IDs.

#### Scenario: package-only fallback filters unrelated same-package outputs

GIVEN a legacy dependency artifact lacks selected producer unit identity
AND a produced artifact index contains outputs for the same package ID with different crate names
WHEN Mantle selects fallback candidates for binding or search paths
THEN candidate matching MUST include the dependency crate name
AND Mantle MUST only raise ambiguous-producer blockers for multiple matching package-and-crate candidates.

### Requirement: Native topology preserves duplicate host unit identity

r[rust_package_planning.native_host_real_unit_identity] Native Rust host topology MUST preserve every selected Cargo host unit identity when planning proc-macro and custom-build artifacts.

#### Scenario: duplicate proc-macro units remain distinct

GIVEN Cargo selects two proc-macro host units with the same package ID, target name, and target kind
WHEN Mantle plans native host artifacts
THEN Mantle MUST keep both selected unit IDs distinct
AND it MUST NOT overwrite one with the other through package/name/kind indexing.

#### Scenario: duplicate custom-build units remain distinct

GIVEN Cargo selects two custom-build host units with the same package ID, target name, and target kind
WHEN Mantle plans native host artifacts and build-script metadata producers
THEN Mantle MUST keep both selected unit IDs distinct
AND consumers MUST bind to the selected producer identity or fail closed on ambiguity.

#### Scenario: host fallback ambiguity blocks before rustc

GIVEN host artifact lookup lacks exact selected producer identity
AND multiple host candidates match the same package ID, target name, and target kind
WHEN Mantle prepares a consuming rustc invocation
THEN Mantle MUST fail before invoking rustc with a deterministic ambiguous-host-producer blocker.
