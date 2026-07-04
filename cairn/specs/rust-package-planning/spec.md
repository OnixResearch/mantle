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

### Requirement: Native Rust manifest and lockfile planning

r[rust_package_planning.native_manifest_lock_planner] Mantle MUST derive Rust workspace, package, dependency, target, and source-lock facts from manifest and lockfile inputs without invoking Cargo.

#### Scenario: workspace package facts are parsed natively

GIVEN a Rust workspace with root and member manifests
WHEN Mantle runs native manifest planning
THEN it MUST emit normalized workspace members, package identities, package metadata, target definitions, dependency tables, and edition/version inheritance facts without calling Cargo.

#### Scenario: lockfile facts are parsed natively

GIVEN a `Cargo.lock` with registry, git, and path package records
WHEN Mantle runs native lockfile planning
THEN it MUST emit source identities, versions, checksums or revisions, and dependency lock edges without calling Cargo.

#### Scenario: unsupported manifest surface fails closed

GIVEN a manifest or lockfile uses Cargo behavior outside Mantle's supported parser subset
WHEN Mantle plans the package
THEN it MUST emit a deterministic unsupported-manifest blocker
AND it MUST NOT silently drop that manifest material.

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

### Requirement: Native Rust executor hardening

r[rust_package_planning.native_executor_hardening] Mantle MUST execute and cache native Rust units only from explicit receipt-bound inputs, artifacts, environment, and toolchain identity.

#### Scenario: cache hit requires full receipt match

GIVEN a prior unit execution receipt and output artifact exist
WHEN Mantle considers reusing the output
THEN it MUST verify unit identity, rustc argument digest, declared input digests, dependency and host artifact digests, environment digest, toolchain identity, and output digest before reporting a cache hit.

#### Scenario: unresolved artifacts block before rustc

GIVEN a unit has dependency or host artifact placeholders
WHEN Mantle prepares rustc execution
THEN every placeholder MUST resolve to a declared existing artifact with matching digest before rustc is invoked.

#### Scenario: failure diagnostics are deterministic

GIVEN rustc execution or preflight validation fails
WHEN Mantle writes the execution receipt
THEN it MUST include a deterministic failure class and redacted diagnostics sufficient for review.

### Requirement: Native build-script runtime parity

r[rust_package_planning.native_build_script_runtime] Mantle MUST execute supported Rust build scripts with an explicit Cargo-compatible environment and typed metadata receipt model.

#### Scenario: build-script environment is declared

GIVEN a selected custom-build host unit runs under Mantle
WHEN Mantle launches the build script
THEN the runtime environment MUST include required package, target cfg, profile, manifest links, and `OUT_DIR` values derived from native planning facts
AND it MUST NOT inherit undeclared ambient host variables.

#### Scenario: build-script metadata is typed

GIVEN a build script writes Cargo metadata lines
WHEN Mantle captures build-script output
THEN it MUST parse supported rustc cfg, rustc env, link lib, link search, rerun, and links metadata into typed receipt fields.

#### Scenario: malformed or unsupported metadata fails closed

GIVEN a build script emits malformed or unsupported metadata
WHEN Mantle parses build-script output
THEN it MUST emit a deterministic build-script-metadata blocker before the dependent target unit runs.

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

### Requirement: Cargo-free Rust planning CLI

r[rust_package_planning.no_cargo_oracle_cli] Mantle MUST expose an explicit Rust planning and topology execution mode that does not invoke Cargo as planner or build orchestrator.

#### Scenario: no-Cargo mode uses native planner

GIVEN a supported Rust workspace
WHEN the user requests Cargo-free Rust planning and execution
THEN Mantle MUST derive planning facts from native manifest, lockfile, feature, unit graph, and source logic
AND it MUST NOT invoke Cargo for metadata, unit graph, or build orchestration.

#### Scenario: accidental Cargo use fails the run

GIVEN Cargo-free mode is requested
WHEN a code path attempts to invoke Cargo
THEN Mantle MUST fail the run with a deterministic cargo-forbidden blocker or audit failure.

#### Scenario: receipt names compatibility class

GIVEN Cargo-free mode completes or blocks
WHEN Mantle emits JSON receipt evidence
THEN the receipt MUST state the Cargo-free mode, supported compatibility class, blockers, and non-claims.

### Requirement: Bounded Cargo-free Rust topology proof

r[rust_package_planning.cargo_free_topology_proof] Mantle MUST provide audit-grade evidence for a bounded Cargo-free Rust planning and topology execution proof on a generated multi-crate path workspace, and MUST NOT claim Mantle/Crunch self-build evidence from this bounded proof.

#### Scenario: proof forbids Cargo planning

GIVEN the Cargo-free proof is launched
WHEN Mantle plans and executes the workspace
THEN the proof environment MUST forbid Cargo metadata, unit graph, and build orchestration use
AND any attempted Cargo invocation MUST fail the proof.

#### Scenario: proof emits durable evidence

GIVEN the Cargo-free proof completes or blocks
WHEN the proof exits
THEN it MUST write durable receipts, command streams, source identity, tool identity, output digests, and blocker summaries to an audit bundle.

#### Scenario: proof validates final outputs

GIVEN the Cargo-free proof reports success
WHEN final outputs are inspected
THEN the audit bundle MUST include output artifact digests and an executable smoke check for the produced proof-fixture binary.
AND the audit bundle MUST name the bounded compatibility class and non-claims.

### Requirement: Cargo-free Rust self-build proof

r[rust_package_planning.cargo_free_self_build_proof] Mantle MUST provide audit-grade evidence for a bounded Cargo-free self-build topology proof that plans and executes the checked-out Mantle workspace without invoking Cargo as planner or build orchestrator.

#### Scenario: self-build proof forbids Cargo

GIVEN the Cargo-free self-build proof is launched
WHEN Mantle plans and executes the checked-out workspace
THEN the proof environment MUST replace Cargo with a failing guard
AND any attempted Cargo metadata, unit-graph, or build-orchestration invocation MUST fail the proof.

#### Scenario: self-build proof binds local source closure

GIVEN the workspace has path, declared vendored registry, and captured git source inputs
WHEN Cargo-free mode derives source and package facts
THEN it MUST derive source identities from checked-in manifests, `Cargo.lock`, declared vendor roots, and captured local source material
AND it MUST NOT use network fetches, ambient Cargo caches, or Cargo metadata output.

#### Scenario: self-build proof emits durable evidence

GIVEN the Cargo-free self-build proof completes or blocks
WHEN the proof exits
THEN it MUST write durable receipts, command streams, source identity, tool identity, output digests, Cargo guard status, blocker summaries, and non-claims to an audit bundle.

#### Scenario: self-build proof validates final CLI output

GIVEN the Cargo-free self-build proof reports success
WHEN final outputs are inspected
THEN the audit bundle MUST include a smoke check of the produced Mantle CLI binary
AND the audit bundle MUST state that the proof is not Crunch fixed-point self-hosting, source-built bootstrap, or release reproducibility evidence.
## ADDED Requirements

### Requirement: Cargo-free Rust fixed-point proof

r[rust_package_planning.cargo_free_fixed_point_proof] Mantle MUST provide audit-grade evidence for a bounded Cargo-free fixed-point proof where host Mantle builds stage1 Mantle, stage1 Mantle builds stage2 Mantle, and the produced stage1/stage2 Mantle binary digests are compared.

#### Scenario: fixed-point proof forbids Cargo in both stages

GIVEN the Cargo-free fixed-point proof is launched
WHEN stage1 and stage2 Mantle builds are executed
THEN each stage MUST replace Cargo with a failing guard
AND any attempted Cargo metadata, unit-graph, or build-orchestration invocation MUST fail the proof.

#### Scenario: stage2 is built by stage1 Mantle

GIVEN stage1 Mantle was produced by a successful Cargo-free topology execution
WHEN the fixed-point proof starts stage2
THEN stage2 MUST be planned and executed by the produced stage1 Mantle binary
AND stage2 MUST use the same Cargo-free native topology rail rather than the original host Mantle binary.

#### Scenario: fixed-point proof emits durable evidence

GIVEN the Cargo-free fixed-point proof completes, blocks, or mismatches
WHEN the proof exits
THEN it MUST write durable preflight metadata, per-stage receipts, command streams, status codes, Cargo guard status, smoke outputs, produced binary paths, BLAKE3 digests, fixed-point status, and non-claims to an audit bundle.

#### Scenario: fixed-point proof compares stage binaries

GIVEN both stage1 and stage2 builds succeed and produce Mantle binaries
WHEN final outputs are inspected
THEN the proof MUST compare stage1 and stage2 Mantle binary BLAKE3 digests
AND it MUST report success only when those digests match.

#### Scenario: fixed-point proof remains bounded

GIVEN the fixed-point proof reports success
WHEN the audit bundle is reviewed
THEN the bundle MUST state that the proof is not Crunch bootstrap, release reproducibility, source-built compiler/toolchain closure, or full Cargo compatibility evidence.

### Requirement: Cargo-free self-build command

r[rust_package_planning.cargo_free_self_build_command] Mantle MUST provide a first-class bounded command that builds a Mantle binary through native Cargo-free Rust topology execution without invoking Cargo as planner or build orchestrator.

#### Scenario: self-build command uses the native Cargo-free topology rail

GIVEN an operator launches `mantle self-build --cargo-free --out <dir>` from a supported Mantle source root
WHEN Mantle performs the build
THEN it MUST execute `rust-plan --no-cargo-oracle --execute-topology` with an explicit execution output root
AND it MUST select exactly one successful `mantle` binary unit from the topology receipt.

#### Scenario: self-build command forbids Cargo

GIVEN the Cargo-free self-build command runs
WHEN planning, topology execution, rustc, or build-script execution attempts to invoke `cargo`
THEN a failing Cargo guard MUST record the attempt
AND the command MUST fail without claiming a Cargo-free build.

#### Scenario: self-build command emits audit evidence

GIVEN the Cargo-free self-build command completes or blocks
WHEN the output directory is inspected
THEN it MUST contain the copied Mantle binary when successful, the full topology receipt, command stderr, status code, smoke output, Cargo guard status, source digest, binary BLAKE3 digest, and a machine-readable summary.

#### Scenario: self-build command remains bounded

GIVEN the Cargo-free self-build command reports success
WHEN the summary is inspected
THEN it MUST state that the result is not Crunch bootstrap, release reproducibility, source-built compiler/toolchain closure, or full Cargo compatibility evidence.

### Requirement: Cargo-free fixed-point command

r[rust_package_planning.cargo_free_fixed_point_command] Mantle MUST provide a first-class command that runs the bounded Cargo-free fixed-point proof without requiring an external proof script or caller-created rustc wrapper.

#### Scenario: command runs both fixed-point stages

GIVEN an operator launches `mantle self-build --cargo-free --fixed-point --out <bundle-dir>` from a supported Mantle source root
WHEN Mantle performs the proof
THEN it MUST build stage1 Mantle through the native Cargo-free topology rail
AND it MUST build stage2 Mantle by invoking the produced stage1 Mantle binary through the same native Cargo-free topology rail.

#### Scenario: command forbids Cargo in both stages

GIVEN the fixed-point command is running
WHEN either stage attempts to invoke Cargo for metadata, unit graph discovery, planning, or build orchestration
THEN the stage-local Cargo guard MUST record the attempt
AND the command MUST fail without claiming a Cargo-free fixed point.

#### Scenario: command owns toolchain compatibility

GIVEN the selected rustc/linker combination needs compatibility handling for Mantle topology execution
WHEN the fixed-point command prepares a stage
THEN it MUST either use reviewed command-owned normalization recorded in the proof bundle
OR fail closed with an actionable toolchain diagnostic
AND it MUST NOT require the caller to provide an untracked external rustc wrapper.

#### Scenario: command emits durable fixed-point evidence

GIVEN the fixed-point command completes, blocks, or detects a mismatch
WHEN the bundle directory is inspected
THEN it MUST contain preflight metadata, per-stage receipts, command streams, status codes, Cargo guard status, smoke outputs, copied Mantle binary paths when produced, per-stage BLAKE3 digests, fixed-point status, and non-claims.

#### Scenario: command reports success only for matching stage binaries

GIVEN both stages produce Mantle binaries
WHEN the fixed-point command compares outputs
THEN it MUST report success only when the stage1 and stage2 Mantle binary BLAKE3 digests match
AND it MUST report a deterministic mismatch status when the digests differ.

#### Scenario: command remains a bounded proof

GIVEN the fixed-point command reports success
WHEN the summary is reviewed
THEN it MUST state that the result is not Crunch bootstrap, release reproducibility, source-built compiler/toolchain closure, or full Cargo compatibility evidence.

### Requirement: Source-built Rust toolchain closure proof

r[rust_package_planning.source_built_toolchain_closure] Mantle MUST provide a separate audit-grade proof before claiming that a Cargo-free self-build or fixed-point run used a source-built compiler/toolchain closure.

#### Scenario: toolchain closure is receipt-bound

GIVEN Mantle is launched in source-built toolchain closure proof mode
WHEN it selects compiler and native toolchain inputs for Rust topology execution
THEN the proof bundle MUST record each compiler, linker, C toolchain, sysroot, crt object, runtime library, and native helper tool with role, source identity, build receipt identity, execution path, BLAKE3 content digest, and trust classification.
AND every non-seed toolchain member MUST have source provenance that is reachable from the proof bundle.

#### Scenario: ambient host toolchain is rejected

GIVEN source-built toolchain closure proof mode is active
WHEN Mantle would use a host `rustc`, Cargo, linker, C compiler, pkg-config, Nix profile tool, PATH helper, or undeclared sysroot member that is not listed in the receipt-bound toolchain closure
THEN Mantle MUST fail before executing the affected Rust unit with a deterministic host-tool-leakage blocker.
AND it MUST NOT report a source-built toolchain closure claim.

#### Scenario: seed exceptions are explicit

GIVEN the proof needs an initial seed or trust root
WHEN the proof bundle is written
THEN every seed exception MUST be named, justified, BLAKE3 hashed, trust-classified, and excluded from the source-built portion of the claim.
AND placeholder providers, missing source-root contracts, or unverified seed metadata MUST fail closed.

#### Scenario: fixed-point stages use the receipt-bound closure

GIVEN source-built toolchain closure proof mode is combined with the Cargo-free fixed-point command
WHEN stage1 and stage2 Mantle builds execute
THEN both stages MUST use the receipt-bound toolchain closure policy rather than ambient host toolchain discovery.
AND the proof MUST report success only when both stages succeed, both stage Cargo guards remain untriggered, both stages record the same closure policy digest, and the stage1/stage2 Mantle binary BLAKE3 digests match.

#### Scenario: bounded non-claims remain visible

GIVEN a source-built toolchain closure proof completes successfully
WHEN the audit bundle is reviewed
THEN it MUST state any remaining non-claims, including whether the proof is not full release reproducibility, not full Cargo compatibility, and not a fully minimized bootstrap trust root when seed exceptions remain.
AND existing Cargo-free fixed-point proofs that lack source-built toolchain closure evidence MUST continue to state `not-source-built-toolchain-closure`.

### Requirement: Rust compiler policy adapter

r[rust_package_planning.compiler_policy_adapter] Mantle MUST model Rust compiler-policy enforcement as an explicit adapter at the Rust unit invocation boundary, separate from the base `rustc` tool identity and separate from rule-provider implementation details.

#### Scenario: Direct Rust unit invocation is adapted

r[rust_package_planning.compiler_policy_adapter.invocation]

- GIVEN a supported Rust unit is ready for direct execution from explicit `unit_derivation_graph` material
- WHEN an operator selects a compiler-policy adapter
- THEN Mantle MUST resolve the base rustc path, unit arguments, unit environment, adapter executable, adapter environment, and adapter identity before invoking the compiler.
- AND Mantle MUST keep filesystem checks, digesting, command construction, and process execution in the imperative shell rather than in pure planning logic.

#### Scenario: Octet adapter remains provider-owned

r[rust_package_planning.compiler_policy_adapter.octet]

- GIVEN the selected compiler-policy provider is Octet
- WHEN Mantle prepares a Rust unit invocation
- THEN Mantle MUST consume Octet-provided adapter artifacts or a provider manifest rather than importing Octet lint implementation internals.
- AND Mantle MUST treat Octet as the authority for lint names, lint levels, standards checks, suppression validation, and diagnostic semantics.

#### Scenario: Operator modes bound enforcement strength

r[rust_package_planning.compiler_policy_adapter.cli]

- GIVEN an operator invokes Rust-plan execution
- WHEN no compiler-policy adapter is selected
- THEN Mantle MUST preserve the current plain rustc behavior and make no Octet compliance claim.
- WHEN audit, deny, or required policy modes are selected
- THEN Mantle MUST record the selected mode and apply its fail-open or fail-closed behavior deterministically.

### Requirement: Compiler policy identity bounds Rust output reuse

r[rust_package_planning.compiler_policy_adapter.cache_identity] Mantle MUST include compiler-policy adapter identity in Rust unit output reuse and execution receipt identity whenever a compiler-policy adapter is selected.

#### Scenario: Raw-rustc output cannot satisfy required policy

r[rust_package_planning.compiler_policy_adapter.cache_identity.raw_rustc_rejected]

- GIVEN a Rust unit output was previously produced without a compiler-policy adapter
- WHEN the same unit is later executed with an Octet-required compiler-policy mode
- THEN Mantle MUST reject reuse of the prior raw-rustc output.
- AND Mantle MUST rebuild or fail closed using the required adapter rather than treating source, dependency, and rustc-argument equality as sufficient.

#### Scenario: Adapter artifacts are receipt-bound

r[rust_package_planning.compiler_policy_adapter.receipts]

- GIVEN a Rust unit execution uses a compiler-policy adapter
- WHEN Mantle records a successful, failed, blocked, or reused execution receipt
- THEN the receipt MUST include the adapter kind, mode, provider-manifest digest when present, driver digest when present, lint-library digest when present, config digest when present, standards-policy digest when present, and a waiver summary when present.
- AND the receipt MUST keep policy-profile compliance claims separate from program-correctness or full-architecture-proof claims.

### Requirement: Compiler policy required mode fails closed

r[rust_package_planning.compiler_policy_adapter.fail_closed] Mantle MUST fail before accepting a Rust unit output when required compiler-policy material is absent, inconsistent, or unable to run.

#### Scenario: Missing adapter material blocks before output acceptance

r[rust_package_planning.compiler_policy_adapter.fail_closed.missing_material]

- GIVEN Octet-required mode is selected
- WHEN the adapter driver, lint library, config, provider manifest, standards policy artifact, or declared digest is missing or mismatched
- THEN Mantle MUST emit a deterministic compiler-policy blocker.
- AND Mantle MUST NOT fall back to plain rustc, weaken the policy mode, or reuse unadapted outputs.

#### Scenario: Standards gate is separate and receipt-bound

r[rust_package_planning.compiler_policy_adapter.standards_gate]

- GIVEN the selected policy profile includes Octet FCIS source-shape standards
- WHEN Mantle accepts a topology-level Rust build result
- THEN Mantle MUST run or verify the declared standards gate and bind its policy artifact, selected scope, pass/fail status, and waiver summary into the topology evidence.
- AND Mantle MUST NOT describe standards-gate success as a formal proof of program correctness.

### Requirement: Compiler policy claims remain bounded

r[rust_package_planning.compiler_policy_adapter.non_claims] Mantle MUST report compiler-policy enforcement as compliance with a named, hashed policy profile rather than as a proof of program correctness or full architecture correctness.

#### Scenario: Compliance wording names explicit scope

r[rust_package_planning.compiler_policy_adapter.non_claims.scope]

- GIVEN Rust unit or topology execution succeeds with a compiler-policy adapter
- WHEN Mantle renders human or JSON evidence
- THEN the evidence MAY claim that the selected unit or topology was accepted under the configured compiler-policy profile.
- AND the evidence MUST NOT claim semantic correctness, memory safety beyond Rust's guarantees, complete FCIS architecture, full Cargo compatibility, or absence of all defects unless separate evidence is present.

### Requirement: Source-built Rust seed closure

r[rust_package_planning.source_built_rust_seed_closure] Mantle MUST NOT treat Rust compiler or Rust sysroot seed exceptions as source-built toolchain closure members unless they are backed by receipt-bound source-built provider metadata.

#### Scenario: Rust provider metadata is complete

GIVEN a source-built Rust compiler/sysroot provider is supplied to a Cargo-free source-built closure proof
WHEN Mantle validates the toolchain closure
THEN the provider MUST identify Rust compiler executables, target standard libraries, host support artifacts, source identities, build receipt identities, executable paths, and BLAKE3 content digests.
AND missing provider metadata, placeholder metadata, digest mismatch, or prebuilt-only provenance MUST fail closed before any source-built closure claim is reported.

#### Scenario: Rust provider separates compiler host and target sysroot evidence

GIVEN a source-built Rust route builds compiler host artifacts and target standard libraries for different triples
WHEN Mantle validates provider metadata, receipts, or closure membership
THEN the provider MUST distinguish the compiler host triple from the target sysroot triple and MUST require receipt-bound artifacts for both host-rustlib and target-rustlib roles.
AND Mantle MUST NOT satisfy the host-rustlib role with only target sysroot artifacts, nor satisfy the target-rustlib role with only compiler-host artifacts.

#### Scenario: Rust seed exceptions remain non-claims

GIVEN the proof still depends on a prebuilt Rust compiler, prebuilt Rust sysroot, or wrapper around an ambient Rust toolchain
WHEN the proof summary is written
THEN Mantle MUST keep the relevant seed exceptions explicit and MUST continue reporting that the run is not a full source-built compiler/toolchain closure.
AND fixed-point binary equality MUST NOT by itself promote those seed exceptions into source-built closure evidence.

### Requirement: Provider-backed source-built Rust closure status

r[rust_package_planning.source_built_toolchain_closure.provider_status] Mantle MUST promote a validated source-built Rust provider into the Cargo-free proof `source_built_toolchain_closure` status instead of reporting the stale `not-source-built-toolchain-closure` non-claim.

#### Scenario: Validated provider supplies closure status

GIVEN a Cargo-free one-shot or fixed-point proof is launched with `--rust-source-provider` and without an explicit `--toolchain-closure` manifest
WHEN Mantle validates the provider metadata and uses the provider Rust compiler for topology execution
THEN the proof summary MUST record `source_built_toolchain_closure.status = provided`, `claim = true`, and the provider policy digest.
AND the proof summary MUST NOT include `not-source-built-toolchain-closure` in `non_claims`.

#### Scenario: Absent provider keeps the non-claim

GIVEN a Cargo-free one-shot or fixed-point proof is launched without `--rust-source-provider` and without an explicit `--toolchain-closure` manifest
WHEN Mantle writes preflight or summary evidence
THEN it MUST keep `source_built_toolchain_closure.status = not-provided`, `claim = false`, and `not-source-built-toolchain-closure`.

#### Scenario: Explicit closure manifest remains authoritative

GIVEN a Cargo-free proof is launched with both `--rust-source-provider` and `--toolchain-closure`
WHEN Mantle validates or enforces the toolchain closure
THEN the explicit closure manifest MUST remain authoritative for `source_built_toolchain_closure` status and policy digest.
AND Mantle MUST still fail on undeclared toolchain input leakage or stage1/stage2 policy digest mismatch.

#### Scenario: Broader proof bounds remain visible

GIVEN a provider-backed Cargo-free fixed-point proof succeeds
WHEN Mantle writes non-claim evidence
THEN it MUST keep unrelated proof bounds including not release reproducibility and not full Cargo compatibility until separate evidence retires those bounds.

### Requirement: Receipt-bound target tool aliases

r[rust_package_planning.source_built_toolchain_closure.target_aliases] Mantle MUST expose target-prefixed native tool names from the receipt-bound toolchain closure manifest before running Cargo-free topology units.

#### Scenario: Target compiler name is declared by the closure

GIVEN a source-built toolchain closure manifest contains an executable member named `x86_64-linux-musl-gcc`
WHEN Mantle constructs the receipt-bound PATH for Cargo-free topology execution
THEN it MUST create a PATH entry named `x86_64-linux-musl-gcc` that dispatches to the declared executable path.
AND the alias MUST be present even when the declared executable file has a different basename.

#### Scenario: Cargo remains guarded

GIVEN a source-built toolchain closure manifest declares an executable member named `cargo`
WHEN Mantle constructs the receipt-bound PATH
THEN it MUST fail before executing any Rust unit.
AND it MUST NOT replace the Cargo guard shim with a toolchain closure alias.

#### Scenario: Alias conflicts fail closed

GIVEN two executable toolchain closure members declare the same alias name but different executable paths
WHEN Mantle constructs the receipt-bound PATH
THEN it MUST fail with a deterministic alias-conflict blocker.

#### Scenario: Target proof does not inherit ambient PATH

GIVEN a musl-target Cargo-free fixed-point proof is launched with `--toolchain-closure`
WHEN a build script requests `x86_64-linux-musl-gcc`
THEN the request MUST resolve only through the receipt-bound PATH alias or fail as a host-tool-leakage/tool-missing blocker.
AND Mantle MUST NOT satisfy the request from `/nix/var/nix/profiles`, `/run/current-system/sw`, rustup, or another ambient PATH entry.

### Requirement: Explicit source-built native closure promotion

r[rust_package_planning.source_built_toolchain_closure.explicit_native_promotion] Mantle MUST promote an explicitly supplied toolchain closure to a source-built toolchain claim only after enforcement proves a zero-seed, all-source-built closure.

#### Scenario: Enforced complete explicit closure claims source-built closure

GIVEN a toolchain closure manifest has no seed exceptions
AND every manifest member is classified as source-built
WHEN Mantle enforces observed Cargo-free execution inputs against that manifest
THEN the resulting source-built toolchain closure status MUST set `claim` to true.
AND it MUST omit `not-source-built-toolchain-closure` from the non-claims.

#### Scenario: Validated-only complete explicit closure does not claim

GIVEN a toolchain closure manifest has no seed exceptions
AND every manifest member is classified as source-built
WHEN Mantle validates the manifest without enforcing observed execution inputs
THEN the resulting source-built toolchain closure status MUST keep `claim` false.
AND it MUST keep `not-source-built-toolchain-closure` in the non-claims.

#### Scenario: Any seed exception preserves the non-claim

GIVEN a toolchain closure manifest contains one or more seed exceptions
WHEN Mantle enforces observed Cargo-free execution inputs against that manifest
THEN the resulting source-built toolchain closure status MUST keep `claim` false.
AND it MUST keep `not-source-built-toolchain-closure` in the non-claims.
AND it MUST report the seed exception count.

#### Scenario: Current native closure frontier is not overclaimed

GIVEN the current source-built Rust provider and receipt-bound musl target aliases
WHEN Mantle attempts a no-seed native closure proof
THEN the evidence MUST record either a successful zero-seed fixed point or the exact deterministic blocker.
AND Mantle MUST NOT relabel Nix clang, glibc, pkg-config, rustup, or other host tools as source-built to satisfy the requirement.

### Requirement: Source-built native closure materialization

r[rust_package_planning.source_built_toolchain_closure.native_materialization] Mantle MUST materialize an explicit source-built native toolchain closure manifest only from digest-bound required members whose provider-root metadata authorizes the root for the requested host or target role, including the source-root musl layout only when its target matches that role's expected triple, and fail closed when the current provider/root lacks host or target native closure inputs.

#### Scenario: Complete provider root emits zero-seed manifest

GIVEN a concrete provider root contains a source-built Rust compiler, host C/linker/runtime/sysroot members, and target-prefixed musl helper members
AND the host and target roots advertise matching source-built native capabilities in their provider metadata
WHEN Mantle materializes a source-built native closure manifest from that root
THEN the manifest MUST contain only source-built members.
AND it MUST contain no seed exceptions.
AND every member MUST include a BLAKE3 content digest and source/build-receipt identity.

#### Scenario: Complete source-root musl host fixture emits zero-seed manifest

GIVEN a musl-host Rust provider identity
AND source-root musl host and target roots expose target-prefixed helpers plus runtime/startup files
WHEN Mantle collects and materializes native closure candidates from those roots
THEN the manifest MUST contain no seed exceptions.
AND host `cc`, host `ld`, and host runtime members MUST point at source-root musl layout paths.
AND every manifest member MUST include a BLAKE3 content digest and source/build-receipt identity.

#### Scenario: Source-root musl can satisfy a musl host root

GIVEN a Rust provider host triple is `x86_64-unknown-linux-musl`
AND a root advertises source-root metadata for `x86_64-linux-musl`
WHEN Mantle collects host native closure members from that root
THEN it MUST use the source-root musl layout for `cc`, `ld`, `crt1.o`, `libgcc_s.so.1`, and `libc.so`.

#### Scenario: Target-only source-root metadata cannot satisfy GNU host root

GIVEN a root advertises source-root metadata for `x86_64-linux-musl`
WHEN Mantle is asked to use that root as the host-native root for a GNU-host Rust provider
THEN it MUST fail before collecting host C/linker/libc/startup/runtime member paths.
AND the diagnostic MUST name the host-root capability mismatch.

#### Scenario: Missing host linker runtime fails closed

GIVEN a provider root contains a source-built Rust compiler but lacks host C/linker/libc/startup/runtime members
WHEN Mantle attempts to materialize a source-built native closure manifest
THEN it MUST fail before writing a claiming manifest.
AND the diagnostic MUST name the missing host native closure surface.

#### Scenario: Missing target helper fails closed

GIVEN a provider root lacks one or more target-prefixed musl helpers
WHEN Mantle attempts to materialize a source-built native closure manifest
THEN it MUST fail before writing a claiming manifest.
AND the diagnostic MUST name the missing target helper roles.

#### Scenario: Fixed-point proof remains authoritative

GIVEN Mantle has materialized a zero-seed native closure manifest
WHEN a Cargo-free fixed-point proof is run with `--toolchain-closure <manifest>`
THEN only the enforced fixed-point summary MAY retire `not-source-built-toolchain-closure`.
AND manifest materialization alone MUST NOT be reported as release reproducibility or full Cargo compatibility evidence.

### Requirement: Selectable Rust source provider route plans

r[rust_package_planning.source_built_toolchain_closure.selectable_rust_source_route] Mantle MUST let Rust source provider materialization select an explicit validated route plan while preserving the existing GNU-host route as the default.

#### Scenario: Default route remains GNU host

GIVEN an operator runs `mantle bootstrap rust-source-provider` without `--route-plan`
WHEN Mantle prepares the Rust source provider materialization
THEN it MUST use the existing recipe-relative `rust-source-plan.ncl` route.
AND that route MUST continue to advertise `host_triple = x86_64-unknown-linux-gnu`.

#### Scenario: Explicit musl-host route validates

GIVEN an operator passes `--route-plan bootstrap/rust-source-musl-host-plan.ncl`
WHEN Mantle loads the Rust source provider route
THEN the route plan MUST validate with `host_triple = x86_64-unknown-linux-musl`.
AND the plan MUST keep `target_triple = x86_64-unknown-linux-musl`.
AND the plan MUST forbid prebuilt Rust.

#### Scenario: Route selection is metadata only

GIVEN the musl-host route plan validates
WHEN no materialized provider output exists
THEN Mantle MUST NOT report a source-built Rust provider claim from the route plan alone.
AND it MUST NOT retire `not-source-built-toolchain-closure`.

### Requirement: Source-root musl Rust provider tools

r[rust_package_planning.source_built_toolchain_closure.source_root_musl_rust_provider_tools] Mantle MUST recognize the source-root musl toolchain layout as a valid tool source for Rust provider bootstrap scripts targeting `x86_64-unknown-linux-musl`.

#### Scenario: Source-root musl prefixes are target aliases

GIVEN a Rust source provider route targets `x86_64-unknown-linux-musl`
WHEN Mantle generates first-stage or Rust-source bootstrap scripts
THEN the scripts MUST search for both `x86_64-unknown-linux-musl-*` and `x86_64-linux-musl-*` target-prefixed tools.
AND they MUST accept both `x86_64-unknown-linux-musl` and `x86_64-linux-musl` `gcc -dumpmachine` values for that target.

#### Scenario: Source-root sysroot is accepted without Nix wrapper metadata

GIVEN the selected target compiler lives under a source-root musl toolchain without `nix-support/orig-libc`
WHEN Mantle configures the Rust source provider target toolchain
THEN it MUST derive the musl sysroot from the selected tool root's `x86_64-linux-musl/` directory.
AND it MUST require libc and CRT files before exporting that target tool configuration.

#### Scenario: Generic host aliases remain host-oriented

GIVEN source-root musl target aliases are available
WHEN Mantle prepares provider bootstrap execution
THEN it MUST NOT replace generic host `cc`, `ld`, or `ld.lld` aliases with musl target tools.
AND target tool exposure MUST remain scoped to target-prefixed names or generated private wrapper directories.

### Requirement: First-stage musl linker wrapper path

r[rust_package_planning.source_built_toolchain_closure.first_stage_musl_linker_wrapper_path] Mantle MUST place generated private musl linker wrappers on PATH before first-stage Rust provider `run_rustc` target links.

#### Scenario: Plain cc resolves to private musl wrapper

GIVEN the generated first-stage script selects the musl target linker wrapper branch
WHEN it creates `$BUILD_DIR/target-linker-bin/cc`
THEN it MUST prepend that private alias directory to PATH before invoking `run_rustc`.
AND a child `rustc` link that asks for plain `cc` MUST resolve inside the generated private alias directory.

#### Scenario: Wrapper remains script-scoped

GIVEN the generated first-stage script prepends the private target linker alias directory
WHEN the script exits
THEN Mantle MUST NOT persist that private `cc` alias as an ambient host tool.
AND target tool exposure MUST remain limited to the generated script process tree.

### Requirement: First-stage musl static-pie normalization

r[rust_package_planning.source_built_toolchain_closure.first_stage_musl_static_pie_normalization] Mantle MUST normalize first-stage source-root musl target wrapper links away from unsupported static PIE mode when the selected seed libc only supports non-PIE static links.

#### Scenario: Rust static-pie request uses source-root static link

GIVEN the generated first-stage musl target wrapper receives `-static-pie`
WHEN it delegates to the source-root musl target GCC
THEN it MUST pass `-static` instead of `-static-pie`.
AND the normalization MUST preserve existing CRT path mapping and runtime object injection.

#### Scenario: Normalization remains private to the first-stage wrapper

GIVEN first-stage source-root musl target link normalization is enabled
WHEN the generated script exits
THEN Mantle MUST NOT expose a global `cc` alias or alter generic host aliases.
AND the normalization MUST apply only through the generated private target wrapper directory.

### Requirement: First-stage musl static CRT normalization

r[rust_package_planning.source_built_toolchain_closure.first_stage_musl_static_crt_normalization] Mantle MUST pair first-stage source-root musl static link mode normalization with a matching non-PIE musl startup object.

#### Scenario: Static-pie downgrade uses non-PIE startup

GIVEN the generated first-stage musl target wrapper receives `-static-pie` and `rcrt1.o`
WHEN it delegates to the source-root musl target GCC as a non-PIE static link
THEN it MUST pass `-static` and private `crt1.o` instead of `-static-pie` and `rcrt1.o`.
AND it MUST continue to pass private `crti.o`, `crtn.o`, `crtbeginS.o`, and `crtendS.o` paths.

#### Scenario: Startup normalization is private to downgraded links

GIVEN the generated first-stage musl target wrapper does not observe `-static-pie`
WHEN it maps musl startup objects
THEN it MUST preserve private `rcrt1.o` mapping for `rcrt1.o` inputs.
AND it MUST NOT expose a global `cc` alias or change generic host aliases.

### Requirement: First-stage musl rustc-driver rlib normalization

r[rust_package_planning.source_built_toolchain_closure.first_stage_musl_rustc_driver_rlib] Mantle MUST normalize the Rust 1.90 `rustc_driver` crate type for the source-root musl compiler-host first-stage build.

#### Scenario: musl compiler host uses an rlib driver

GIVEN the generated first-stage Rust provider script is building `run_rustc` with `RUSTC_HOST_TRIPLE=x86_64-unknown-linux-musl`
WHEN the extracted Rust source contains `compiler/rustc_driver/Cargo.toml` with `crate-type = ["dylib"]`
THEN Mantle MUST rewrite that crate type to `crate-type = ["rlib"]` before invoking the `run_rustc` compiler-host build.
AND the rewrite MUST be private to the generated first-stage source tree.

#### Scenario: non-musl compiler hosts keep upstream driver shape

GIVEN the generated first-stage Rust provider script is building a compiler host other than `x86_64-unknown-linux-musl`
WHEN it prepares the extracted Rust source
THEN Mantle MUST NOT rewrite `compiler/rustc_driver/Cargo.toml` for that host route.
AND generic host aliases such as `cc`, `ld`, and `ld.lld` MUST remain host-oriented.

#### Scenario: unexpected driver manifest fails closed

GIVEN the generated first-stage Rust provider script is building the source-root musl compiler-host route
WHEN `compiler/rustc_driver/Cargo.toml` is missing or lacks either `crate-type = ["dylib"]` or an already-normalized `crate-type = ["rlib"]`
THEN Mantle MUST abort the first-stage build with a deterministic diagnostic before invoking `run_rustc` for the compiler host.

### Requirement: First-stage musl proc-macro runtime visibility

r[rust_package_planning.source_built_toolchain_closure.first_stage_musl_proc_macro_runtime] Mantle MUST make source-root musl runtime libraries visible to first-stage musl-host proc macros without exposing generic host aliases globally.

#### Scenario: musl proc macros load source-root libc

GIVEN the generated first-stage Rust provider script is building `run_rustc` with `RUSTC_HOST_TRIPLE=x86_64-unknown-linux-musl`
WHEN mrustc's `run_rustc` build loads host proc-macro shared objects such as `tracing_attributes`
THEN Mantle MUST place the source-root musl `libc.so` in the private first-stage runtime directory.
AND the `run_rustc` compiler-host build MUST search that private runtime directory before its build libdir when loading proc macros.

#### Scenario: runtime normalization fails closed

GIVEN the generated first-stage Rust provider script is building the source-root musl compiler-host route
WHEN the source-root musl `libc.so` is missing or `run_rustc/Makefile` lacks the expected `RUSTC_ENV_VARS` `LD_LIBRARY_PATH` line
THEN Mantle MUST abort the first-stage build with a deterministic diagnostic before invoking `run_rustc` for the compiler host.

#### Scenario: generic host aliases remain host-oriented

GIVEN the generated first-stage Rust provider script prepares musl proc-macro runtime visibility
WHEN the private runtime search path is installed
THEN Mantle MUST NOT replace global `cc`, `ld`, `ld.lld`, Cargo, or host linker aliases with source-root musl target tools.
AND ambient Cargo rustc wrappers MUST be scrubbed before first-stage build commands run.

### Requirement: First-stage musl stage2 rustc probe runtime visibility

r[rust_package_planning.source_built_toolchain_closure.first_stage_musl_stage2_rustc_probe_runtime] Mantle MUST make the private source-root musl runtime visible to first-stage stage2 Cargo `rustc` probes.

#### Scenario: stage2 Cargo inherits rustc runtime environment

GIVEN the generated first-stage Rust provider script is building `run_rustc` with `RUSTC_HOST_TRIPLE=x86_64-unknown-linux-musl`
WHEN mrustc's `run_rustc` stage2 standard-library Cargo build probes `rustc -vV` through `rustc_proxy.sh`
THEN the `CARGO_ENV_STAGE2_STD` Makefile environment MUST include `$(RUSTC_ENV_VARS)`.
AND the probed musl-host rustc MUST see the same private source-root musl runtime search path as later compiler-host Cargo builds.

#### Scenario: unexpected stage2 env line fails closed

GIVEN the generated first-stage Rust provider script is building the source-root musl compiler-host route
WHEN `run_rustc/Makefile` lacks either the expected original `CARGO_ENV_STAGE2_STD` line or an already-normalized line that includes `$(RUSTC_ENV_VARS)`
THEN Mantle MUST abort the first-stage build with a deterministic diagnostic before invoking `run_rustc` for the compiler host.

#### Scenario: runtime scope stays private

GIVEN the generated first-stage Rust provider script normalizes stage2 Cargo rustc probes
WHEN the private runtime search path is installed
THEN Mantle MUST NOT replace global `cc`, `ld`, `ld.lld`, Cargo, or host linker aliases with source-root musl target tools.

### Requirement: First-stage musl stage2 prefix runtime visibility

r[rust_package_planning.source_built_toolchain_closure.first_stage_musl_stage2_prefix_runtime] Mantle MUST include the stage2 rustc prefix runtime directory in source-root musl first-stage Cargo probes.

#### Scenario: stage2 rustc probe can load prefix runtime libraries

GIVEN the generated first-stage Rust provider script is building `run_rustc` with `RUSTC_HOST_TRIPLE=x86_64-unknown-linux-musl`
WHEN mrustc's `run_rustc` stage2 standard-library Cargo build probes `$(BINDIR_2)rustc` through `rustc_proxy.sh`
THEN the `RUSTC_ENV_VARS` `LD_LIBRARY_PATH` entry MUST include the private source-root musl runtime directory, `$(PREFIX_2)lib`, and `$(LIBDIR)` in that order.
AND the `CARGO_ENV_STAGE2_STD` Makefile environment MUST inherit `$(RUSTC_ENV_VARS)`.

#### Scenario: unexpected runtime line fails closed

GIVEN the generated first-stage Rust provider script is building the source-root musl compiler-host route
WHEN `run_rustc/Makefile` lacks either the expected original `RUSTC_ENV_VARS += LD_LIBRARY_PATH=$(abspath $(LIBDIR))` line or an already-normalized line including `$(PREFIX_2)lib`
THEN Mantle MUST abort the first-stage build with a deterministic diagnostic before invoking `run_rustc` for the compiler host.

#### Scenario: runtime scope stays private

GIVEN the generated first-stage Rust provider script normalizes stage2 rustc prefix runtime visibility
WHEN the private runtime search path is installed
THEN Mantle MUST NOT replace global `cc`, `ld`, `ld.lld`, Cargo, or host linker aliases with source-root musl target tools.

### Requirement: First-stage musl libgcc_eh unwind binding

r[rust_package_planning.source_built_toolchain_closure.first_stage_musl_libgcc_eh_unwind] Mantle MUST bind first-stage source-root musl target `-lunwind` requests to a private target GCC unwind archive, preferring `libgcc_eh.a` when it is present.

#### Scenario: Unwind archive is selected from libgcc_eh

GIVEN the generated first-stage musl target wrapper selects a source-root target GCC CRT directory
WHEN it prepares private runtime libraries
THEN it MUST choose `libgcc_eh.a` as the unwind archive when that file exists.
AND it MUST copy the selected unwind archive to the private runtime directory as `libunwind.a`.

#### Scenario: Libgcc aliases remain available

GIVEN the generated first-stage musl target wrapper prepares private runtime libraries
WHEN Rust target links request libgcc-style aliases
THEN Mantle MUST continue to expose `libgcc.a` through the private runtime directory.
AND the private unwind binding MUST NOT expose generic target libraries globally.

#### Scenario: Libgcc fallback remains available for folded toolchains

GIVEN the selected target GCC CRT directory lacks `libgcc_eh.a`
AND `libgcc.a` is present
WHEN the generated first-stage musl target wrapper prepares private runtime libraries
THEN it MAY use `libgcc.a` as the private `libunwind.a` source.
AND later link failure from missing unwind symbols MUST remain a fail-closed build result.

### Requirement: Source-built Rust bootstrap patch-plan boundary

r[rust_package_planning.source_built_toolchain_closure.bootstrap_patch_plan_boundary] Mantle MUST isolate source-built Rust compiler-bootstrap repair decisions in a deterministic patch-plan boundary before provider materialization mutates source trees or emits generated shell.

#### Scenario: patch-plan core derives operations from explicit facts

GIVEN Mantle selects a source-built Rust provider route with explicit Rust version, mrustc version, source identities, compiler-host triple, provider-target triple, and route capabilities
WHEN Mantle determines compiler-bootstrap repairs for that route
THEN it MUST derive an ordered patch plan from those explicit facts.
AND the patch-plan derivation MUST avoid filesystem reads, process execution, environment reads, clocks, and mutation.
AND the patch plan MUST carry a deterministic BLAKE3 digest over canonical plan inputs and ordered operation outputs.

#### Scenario: provider shell applies operations fail-closed

GIVEN a source-built Rust provider materialization receives a patch plan
WHEN the provider shell applies source, makefile, wrapper, relink, helper-object, or metadata operations
THEN it MUST apply the operations in order.
AND it MUST verify every expected anchor or source identity before mutation.
AND it MUST fail closed with deterministic diagnostics when an expected anchor, source identity, or supported operation kind is missing.

#### Scenario: patch-plan evidence is receipt-bound

GIVEN a source-built Rust provider materialization applies a patch plan
WHEN Mantle records provider evidence or metadata
THEN it MUST bind the patch-plan input digest, output digest, operation summaries, route facts, compiler versions, and source identities.
AND downstream claims about the provider MUST be reviewable without reading ad-hoc generated shell fragments.

### Requirement: Source-built Rust provider contract independence

r[rust_package_planning.source_built_toolchain_closure.provider_contract_independence] Mantle MUST keep downstream Rust planning, topology execution, and self-build consumers coupled to a stable source-built toolchain provider contract rather than compiler-bootstrap implementation details.

#### Scenario: downstream consumers use normalized provider capabilities

GIVEN Mantle has materialized a source-built Rust provider
WHEN native Rust planning, topology execution, or self-build code consumes that provider
THEN those consumers MUST use normalized provider capabilities such as executable paths, host and target triples, sysroot root, proc-macro runtime support, static and dynamic linking policy, source provenance, and provider digests.
AND those consumers MUST NOT branch on mrustc, minicargo, LLVM, or generated-wrapper repair internals.

#### Scenario: compiler-bootstrap repairs stay behind the provider boundary

GIVEN a compiler-bootstrap repair changes for mrustc, minicargo, LLVM, dynamic musl `rustc`, proc-macro loading, or runtime wrapper behavior
WHEN the source-built Rust provider still satisfies the same normalized provider contract
THEN Mantle MUST NOT require native Rust planning, topology execution, or self-build consumer code changes solely because that repair changed.
AND any provider-contract migration MUST be explicit, versioned, and receipt-bound.

### Requirement: Source-built Rust provider fixed-point handoff

r[rust_package_planning.source_built_toolchain_closure.provider_fixed_point] Mantle MUST run provider-backed Cargo-free one-shot and fixed-point proof stages with the explicit receipt-bound source-built Rust/native toolchain closure when that closure is supplied.

#### Scenario: compatibility probes use the receipt-bound closure

GIVEN a Cargo-free proof is launched with `--rust-source-provider` and `--toolchain-closure`
WHEN Mantle probes whether the selected Rust compiler accepts proof-required rustc flags
THEN the probe MUST run with the receipt-bound toolchain PATH aliases derived from the explicit closure.
AND the probe MUST NOT discover C compilers, linkers, or helper tools from ambient PATH entries.
AND a failed probe MUST produce a deterministic source-built closure blocker instead of creating a compatibility wrapper outside the closure.

#### Scenario: source-root unwind archive is declared

GIVEN Mantle materializes a native closure from the source-root musl provider layout
WHEN the source-root GCC runtime contains an unwind archive used to satisfy Rust `-lunwind` links
THEN the manifest MUST record that unwind archive as a source-built runtime member with BLAKE3 digest and source/build receipt identity.
AND the receipt-bound C compiler alias MUST expose only that declared archive as `libunwind.a` for Rust linker compatibility.

#### Scenario: fixed-point evidence stays bounded

GIVEN a provider-backed Cargo-free one-shot or fixed-point proof reaches a blocker or succeeds
WHEN Mantle writes the proof summary and evidence
THEN the evidence MUST report whether the explicit closure was enforced, the policy digest used by each completed stage, and the exact blocker if any stage fails.
AND the evidence MUST NOT claim release reproducibility, full Cargo compatibility, or a broader bootstrap proof from provider-backed fixed-point evidence alone.

### Requirement: Provider fixed-point release verifier

r[rust_package_planning.provider_fixed_point_release_verifier] Mantle MUST let release verification validate a provider-backed Cargo-free fixed-point proof bundle as bounded release-adjacent evidence without upgrading the release reproducibility claim.

#### Scenario: valid provider fixed-point proof is reported separately

GIVEN an operator runs `mantle release verify <bundle-dir> --provider-fixed-point-proof <proof-dir>`
WHEN the supplied proof bundle contains successful stage1 and stage2 summaries with matching Mantle binary BLAKE3 digests, absent Cargo markers, successful smoke checks, successful stage receipts, and enforced source-built closure policy digests
THEN release verification MUST report the provider fixed-point proof status separately from reproducibility and deterministic release eligibility.
AND it MUST report the closure policy digest and matching stage binary digest.

#### Scenario: required provider fixed-point proof fails closed

GIVEN an operator runs `mantle release verify <bundle-dir> --require-provider-fixed-point-proof`
WHEN the provider fixed-point proof path is missing, malformed, has failed stages, has mismatched stage binary digests, lacks enforced source-built closure status, or omits bounded non-claims
THEN release verification MUST fail closed with deterministic blockers that name the invalid proof condition.

#### Scenario: fixed-point proof stays bounded

GIVEN a provider fixed-point proof verifies successfully during release verification
WHEN Mantle renders human or JSON release verification output
THEN the result MUST keep release reproducibility and full Cargo compatibility non-claims visible.
AND it MUST NOT set deterministic release eligibility or reproducibility status from fixed-point proof evidence alone.

### Requirement: Bundle-local provider fixed-point release evidence

r[rust_package_planning.bundle_provider_fixed_point_release_evidence] Mantle MUST let release evidence bundles carry a provider-backed Cargo-free fixed-point proof as bounded, verifiable release-adjacent evidence.

#### Scenario: release creation packages validated provider fixed-point proof

GIVEN an operator runs `mantle release create --provider-fixed-point-proof <proof-dir>`
WHEN the proof directory is a valid provider-backed Cargo-free fixed-point proof bundle
THEN release creation MUST copy the proof into the release evidence bundle and record a manifest artifact with its relative path, BLAKE3 digest, size, and bounded evidence role.
AND release creation MUST fail closed before writing trusted manifest evidence when the proof is malformed, mismatched, missing enforced source-built closure status, or missing bounded non-claims.

#### Scenario: release verification uses bundled proof when required

GIVEN a release evidence bundle records a provider fixed-point proof artifact
WHEN an operator runs `mantle release verify <bundle-dir> --require-provider-fixed-point-proof` without an external proof path
THEN release verification MUST validate the bundled proof artifact and report its status, artifact digest, closure policy digest, and matching stage binary digest.
AND release verification MUST fail closed with deterministic blockers when the manifest omits the proof artifact or the bundled proof is invalid.

#### Scenario: external proof override remains explicit

GIVEN a release evidence bundle records a provider fixed-point proof artifact
WHEN an operator also supplies `--provider-fixed-point-proof <proof-dir>`
THEN release verification MUST validate the explicitly supplied proof path and report that the external proof source was used.
AND the bundled proof artifact MUST remain recorded in the manifest output without being confused with the external override result.

#### Scenario: bundled fixed-point proof stays bounded

GIVEN release verification validates a bundled provider fixed-point proof
WHEN Mantle renders human or JSON release verification output
THEN the result MUST keep release reproducibility and full Cargo compatibility non-claims visible.
AND it MUST NOT set reproducibility status, deterministic release eligibility, or release artifact digest matching from provider fixed-point proof evidence alone.

### Requirement: Rust-plan remains an explicit verification lane [r[rust_package_planning.verification_lane_boundary]]

Mantle MUST keep native `rust-plan` execution as an explicit bounded verification lane until separate promotion evidence justifies using it as a default project-build path, and MUST keep rust-plan claims machine-readable and narrower than the evidence.

#### Scenario: Default project builds do not silently use rust-plan [r[rust_package_planning.verification_lane_boundary.scenario.default-build]]

- GIVEN a Mantle project declares a Rust package for the supported offline Cargo build lane
- WHEN the operator runs `mantle build .#name` without an explicit native-planner opt-in
- THEN Mantle MUST use the declared project build workflow rather than silently invoking native rust-plan topology execution
- AND the resulting report MUST NOT claim Cargo-free execution.

#### Scenario: Explicit rust-plan emits bounded evidence [r[rust_package_planning.verification_lane_boundary.scenario.explicit]]

- GIVEN the operator explicitly invokes `mantle rust-plan` with a supported execution flag
- WHEN native planning or topology execution succeeds
- THEN Mantle MUST emit deterministic receipt evidence that names the supported graph, source facts, toolchain identity, output digests, and claim class
- AND the claim MUST remain bounded to the explicit units or topology that were executed.

#### Scenario: Unsupported native planner surfaces fail closed [r[rust_package_planning.verification_lane_boundary.scenario.unsupported]]

- GIVEN a Rust workspace requires Cargo behavior outside Mantle's supported native-planner surface
- WHEN rust-plan planning or execution evaluates that workspace
- THEN Mantle MUST emit deterministic blockers naming the unsupported surface
- AND it MUST NOT invoke Cargo as hidden build orchestration while claiming Cargo-free success.

#### Scenario: Promotion requires current evidence [r[rust_package_planning.verification_lane_boundary.scenario.promotion]]

- GIVEN a future change proposes to make native rust-plan execution a default or broader project-build path
- WHEN that change is reviewed
- THEN it MUST cite current compatibility-rail evidence, positive and negative fixture coverage, bounded report wording, and validation receipts
- AND it MUST preserve non-claims for full Cargo compatibility, compiler correctness, release reproducibility, and bootstrap correctness unless separate evidence proves them.

### Requirement: Representative Rust compatibility workspace rail [r[rust_package_planning.compatibility_workspace_rail]]

Mantle MUST maintain a representative Rust workspace rail before making broad practical Rust project support claims, and MUST distinguish sandboxed offline Cargo results from native rust-plan results for that rail.

#### Scenario: Rail fixture covers practical Rust surfaces [r[rust_package_planning.compatibility_workspace_rail.scenario.fixture]]

- GIVEN Mantle reports representative Rust project-build support
- WHEN the compatibility rail fixture is inspected
- THEN the fixture MUST include a runnable binary, local library dependency, vendored registry dependency, proc-macro host artifact, build-script metadata surface, and explicit source-closure facts
- AND the fixture MUST run without external network or ambient Cargo cache access in fast validation.

#### Scenario: Offline Cargo lane proves practical build for the fixture [r[rust_package_planning.compatibility_workspace_rail.scenario.offline-cargo]]

- GIVEN the representative fixture has complete declared source material
- WHEN the offline Cargo project build lane validates it
- THEN Mantle MUST build the fixture in the sandbox, run the declared binary smoke, and emit report evidence binding source closure, toolchain, output, and artifact attestation
- AND the evidence MUST identify the result as Cargo-orchestrated inside Mantle rather than Cargo-free native execution.

#### Scenario: Rust-plan lane reports bounded success or blockers [r[rust_package_planning.compatibility_workspace_rail.scenario.rust-plan]]

- GIVEN the representative fixture is evaluated through explicit `mantle rust-plan` topology execution
- WHEN native planning or execution reaches an unsupported surface
- THEN Mantle MUST emit a deterministic blocker naming that surface and preserving partial evidence when available
- AND it MUST NOT fall back to Cargo while claiming Cargo-free success.

#### Scenario: Compatibility claims are evidence-scoped [r[rust_package_planning.compatibility_workspace_rail.scenario.claims]]

- GIVEN a task, README, release note, or status reply cites the representative rail
- WHEN it states what Mantle can build
- THEN the claim MUST identify which lane passed, which fixture was inspected, and which evidence file or command output proves it
- AND it MUST NOT generalize that result to full Cargo compatibility, compiler correctness, release reproducibility, or bootstrap correctness.

### Requirement: Bundle-local deterministic release proof

r[rust_package_planning.bundle_deterministic_release_proof] Mantle MUST let release evidence bundles carry deterministic-release proof receipts as bounded, verifiable bundle-local evidence.

#### Scenario: release reproduce attaches deterministic proof artifacts

GIVEN an operator runs `mantle release reproduce <bundle-dir>` with deterministic proof runs enabled
WHEN the repeated rebuild proof succeeds
THEN Mantle MUST write the deterministic build proof receipt and deterministic sandbox isolation evidence into the release evidence bundle and record manifest artifacts with relative paths, BLAKE3 digests, sizes, and bounded evidence roles.
AND Mantle MUST fail closed before recording trusted manifest evidence when a proof artifact is missing, malformed, or inconsistent with the rebuilt artifact digest set.

#### Scenario: release verification uses bundled deterministic proof when required

GIVEN a release evidence bundle records deterministic proof artifacts
WHEN an operator runs `mantle release verify <bundle-dir> --require-deterministic-release` without external deterministic proof paths
THEN release verification MUST validate the bundled deterministic proof receipt and sandbox isolation evidence and report deterministic-release eligibility from that proof.
AND verification MUST fail closed with deterministic blockers when the manifest omits required proof artifacts, any bundled proof artifact is missing, or proof digests do not match manifest evidence.

#### Scenario: external deterministic proof override remains explicit

GIVEN a release evidence bundle records deterministic proof artifacts
WHEN an operator supplies `--deterministic-proof <path>` and `--deterministic-sandbox-isolation-evidence <path>`
THEN release verification MUST validate the explicitly supplied proof paths and report that external deterministic proof evidence was used.
AND the bundle-local proof artifacts MUST remain recorded in manifest output without being confused with the external override result.

#### Scenario: bundled deterministic proof stays bounded

GIVEN release verification validates bundled deterministic proof artifacts
WHEN Mantle renders human or JSON release verification output
THEN the result MAY report deterministic-release eligibility for the packaged artifact set described by the proof.
AND it MUST NOT claim compiler correctness, full bootstrap reproducibility, full Cargo compatibility, deploy success, or physical-target determinism unless separate evidence proves those claims.

### Requirement: Provider fixed-point release artifact binding

r[rust_package_planning.provider_fixed_point_release_artifact_binding] Mantle MUST bind provider-backed Cargo-free fixed-point proof evidence to the packaged release artifact digest before reporting provider-backed release artifact evidence.

#### Scenario: release creation packages only matching provider proof evidence

GIVEN an operator runs `mantle release create --provider-fixed-point-proof <proof-dir>` with one or more `--binary` release artifacts
WHEN the provider fixed-point proof validates successfully
THEN release creation MUST require the proof's fixed-point stage binary BLAKE3 digest to match at least one bundled release binary artifact digest.
AND release creation MUST fail closed before writing trusted manifest evidence when no bundled release binary matches the provider proof stage binary digest.

#### Scenario: release verification rejects mismatched provider proof evidence

GIVEN a release evidence bundle records a provider fixed-point proof artifact or an operator supplies `--provider-fixed-point-proof <proof-dir>`
WHEN `mantle release verify --require-provider-fixed-point-proof` validates that proof
THEN release verification MUST require the proof's fixed-point stage binary BLAKE3 digest to match at least one release binary recorded in the manifest.
AND release verification MUST fail closed with a deterministic blocker when the proof is otherwise valid but does not match the release artifact set.

#### Scenario: matched provider proof reports release artifact identity

GIVEN provider fixed-point proof evidence matches a packaged release binary artifact
WHEN Mantle renders human or JSON release verification output
THEN the output MUST identify the matched release artifact relative path and BLAKE3 digest alongside the provider proof status.
AND the output MUST keep deterministic-release, reproducibility, bootstrap, compiler-correctness, and full-Cargo-compatibility claims separate unless separate evidence proves them.

### Requirement: Source-built provider AWS-LC memcmp guard handling

r[rust_package_planning.source_built_toolchain_closure.aws_lc_memcmp_guard] Mantle MUST handle AWS-LC's GCC PR95189 `memcmp` compiler guard honestly when running provider-backed Cargo-free topology execution under a source-built native closure.

#### Scenario: Guard failure is a deterministic blocker

- GIVEN provider-backed Cargo-free topology execution runs an AWS-LC build script with a receipt-bound source-built C compiler
- WHEN AWS-LC's `memcmp_invalid_stripped_check` reports the selected compiler is affected by GCC PR95189
- THEN Mantle MUST report a deterministic compiler-guard blocker naming `aws-lc-sys`, the selected compiler identity, and the guard diagnostic.
- AND Mantle MUST NOT report provider fixed-point success or source-built release artifact evidence from that blocked run.

#### Scenario: Safe compiler route is receipt-bound

- GIVEN a source-built native closure provides more than one C compiler route
- WHEN Mantle selects a compiler for AWS-LC C build-script execution
- THEN the selected compiler MUST be declared in the closure manifest with role, source identity, build receipt identity, execution path, and BLAKE3 digest.
- AND successful AWS-LC guard handling MUST record the selected compiler route in the unit or proof receipt.

#### Scenario: Guard bypasses are forbidden

- GIVEN AWS-LC guard handling is required for a provider-backed proof
- WHEN Mantle prepares build-script environment or compiler selection
- THEN it MUST NOT bypass the guard by spoofing `HOST`/`TARGET`, forwarding undeclared ambient `CC`, forwarding arbitrary `CFLAGS`, suppressing the guard, or falling back to an undeclared host compiler.
- AND any unavailable safe route MUST leave the proof blocked with bounded non-claims instead of weakening the source-built closure claim.

### Requirement: Source-built provider snix sandbox-shell compile environment

r[rust_package_planning.source_built_toolchain_closure.snix_sandbox_shell_compile_env] Mantle MUST handle vendored `snix-build` sandbox-shell compile-time defaults without requiring undeclared ambient `SNIX_BUILD_SANDBOX_SHELL` during provider-backed Cargo-free topology execution under a source-built native closure.

#### Scenario: Missing compile-time shell env is a placeholder, not a proof claim

GIVEN provider-backed Cargo-free topology execution compiles vendored `snix-build` or Mantle diagnostics with `SNIX_BUILD_SANDBOX_SHELL` absent from the child environment
WHEN the code needs a compile-time sandbox-shell default
THEN Mantle MUST compile using an explicit placeholder default rather than failing at `env!("SNIX_BUILD_SANDBOX_SHELL")`.
AND Mantle MUST NOT claim the placeholder is a source-built sandbox shell or source-built runtime sandbox execution evidence.

#### Scenario: Runtime shell selection remains explicit

GIVEN a compiled Mantle binary later performs sandboxed builds
WHEN runtime `SNIX_BUILD_SANDBOX_SHELL` is set to a non-placeholder executable
THEN runtime shell selection MUST prefer that explicit runtime value over the compile-time placeholder.
AND absence of a runtime shell MUST remain bounded by existing runtime discovery or failure behavior rather than provider fixed-point proof success.

#### Scenario: Provider proof frontier is rerun honestly

GIVEN the compile-time sandbox-shell blocker has been removed
WHEN the provider-backed fixed-point proof is rerun from current code
THEN Mantle MUST record whether fixed-point succeeds or the next deterministic blocker appears.
AND any blocked run MUST retain bounded non-claims instead of reporting provider fixed-point release artifact evidence.

### Requirement: Source-built provider Mantle binary warning frontier

r[rust_package_planning.source_built_toolchain_closure.mantle_bin_warning_frontier] Mantle MUST keep provider-backed Cargo-free topology execution from failing the native Mantle binary on unused-item warnings caused by test-only helpers or intentionally dormant provider/front-end/native-planner surfaces leaking into non-test compilation.

#### Scenario: Test-only imports do not warn in native binary execution

GIVEN provider-backed Cargo-free topology execution compiles the native Mantle binary
WHEN a helper import is used only by `#[cfg(test)]` code
THEN Mantle MUST scope that helper import to the test code rather than exposing it to the non-test binary build.
AND Mantle MUST NOT suppress the warning globally to hide unrelated unused imports.

#### Scenario: Test-only constants do not warn in native binary execution

GIVEN provider-backed Cargo-free topology execution compiles the native Mantle binary
WHEN a helper constant is used only by `#[cfg(test)]` code
THEN Mantle MUST scope that helper constant to the test code rather than exposing it to the non-test binary build.
AND Mantle MUST NOT turn the constant into proof evidence or a source-built closure claim unless runtime proof receipts actually use it.

#### Scenario: Dormant surfaces use scoped allowances

GIVEN a provider/front-end/native-planner surface is intentionally compiled but not wired into the current native Mantle binary path
WHEN that surface would emit `dead_code` warnings during provider-backed topology execution
THEN Mantle MAY apply item- or module-scoped `dead_code` allowances for that dormant surface.
AND Mantle MUST NOT apply crate-wide unused-code suppression that would hide unrelated warning regressions.

#### Scenario: Provider proof frontier is rerun honestly

GIVEN the Mantle binary warning frontier has been addressed
WHEN the provider-backed fixed-point proof is rerun from current code
THEN Mantle MUST record whether fixed-point succeeds or the next deterministic blocker appears.
AND any blocked run MUST retain bounded non-claims instead of reporting provider fixed-point release artifact evidence.

### Requirement: Source-built provider native static-PIE CRT normalization

r[rust_package_planning.source_built_toolchain_closure.native_static_pie_crt] Mantle MUST normalize provider-backed Cargo-free native topology links away from unsupported source-root musl static-PIE CRT inputs when the receipt-bound closure supplies only non-PIE static CRT material.

#### Scenario: Receipt-bound alias replaces static-PIE startup object

GIVEN a provider-backed Cargo-free topology unit links through the receipt-bound C compiler alias
WHEN Rust passes `rcrt1.o` or `-static-pie` directly or through a readable linker response file
THEN Mantle MUST pass the manifest-declared private `crt1.o` to the source-root musl C compiler instead of the original `rcrt1.o`.
AND Mantle MUST rewrite readable response files under the alias runtime directory before forwarding them to the source-root C compiler.
AND Mantle MUST append `-no-pie` when `-static-pie` is downgraded to `-static` so source-root GCC defaults cannot keep the final link in PIE mode.
AND Mantle MUST keep the replacement under the alias runtime directory rather than using an ambient sysroot path.

#### Scenario: CRT facts are manifest-bound

GIVEN an explicit source-built toolchain closure manifest is supplied
WHEN Mantle prepares the receipt-bound C compiler alias for native topology execution
THEN Mantle MUST derive the private CRT object from exactly one declared target CRT closure member.
AND Mantle MUST fail closed instead of guessing a CRT path when the declared target CRT member is missing or ambiguous.

#### Scenario: Provider proof frontier is rerun honestly

GIVEN native static-PIE CRT normalization has been addressed
WHEN the provider-backed fixed-point proof is rerun from current code
THEN Mantle MUST record whether fixed-point succeeds or the next deterministic blocker appears.
AND any blocked run MUST retain bounded non-claims instead of reporting provider fixed-point release artifact evidence.

### Requirement: Source-root host/target Rust topology

r[rust_package_planning.source_root_host_target_topology] Mantle MUST plan and execute Cargo-free Rust topology units with explicit host, target, and host-dependency roles when a source-root or receipt-bound toolchain closure is selected.

#### Scenario: host-dependency units use the host role

GIVEN a Cargo-free topology contains a host custom-build unit whose support crates are also present in the target dependency graph
WHEN Mantle derives native unit and artifact facts
THEN those support crates MUST be represented as host-dependency units for the host execution triple before the custom-build unit consumes them.
AND Mantle MUST NOT satisfy the host custom-build unit from a target-built support artifact.

#### Scenario: target units remain target scoped

GIVEN the same package graph contains target libraries or binaries
WHEN Mantle derives their dependency artifacts
THEN target units MUST keep the requested target triple and target toolchain-closure policy.
AND Mantle MUST NOT rewrite target-library dependencies to host-dependency artifacts unless the dependency edge is a host execution edge.

#### Scenario: artifact identity is role and triple sensitive

GIVEN a package target can appear in more than one role or triple
WHEN Mantle records produced and consumed artifacts
THEN artifact identity MUST include package identity, target identity, role, selected triple, source digest, feature set, metadata hash, and toolchain-policy digest.
AND same-package host and target artifacts MUST NOT collide in receipt maps, output paths, or dependency lookup.

#### Scenario: mismatched artifacts fail before rustc

GIVEN a unit consumes an artifact whose role, triple, source digest, metadata hash, or toolchain-policy digest does not match the planned dependency edge
WHEN Mantle prepares execution for that unit
THEN Mantle MUST fail before invoking `rustc` with a deterministic artifact-mismatch blocker.
AND the blocker MUST identify the expected and observed role and triple without searching Cargo target directories or ambient caches.

#### Scenario: receipts expose the split

GIVEN role-aware topology execution completes or blocks
WHEN Mantle writes topology receipts or blocker summaries
THEN the evidence MUST record each unit's role, selected triple, toolchain-policy digest, produced artifact digest when present, and consumed artifact roles.
AND any Cargo-free or source-root proof claim MUST be bounded to the recorded role-aware evidence.

### Requirement: Wrapperless source-root fixed-point proof

r[rust_package_planning.wrapperless_source_root_fixed_point] Mantle MUST run the Cargo-free fixed-point command from declared source-root or toolchain-closure inputs without requiring a caller-created Nix, rustup, or otherwise untracked rustc wrapper.

#### Scenario: command-owned normalization is receipt-bound

GIVEN an operator launches the Cargo-free fixed-point command with a declared source-root provider or toolchain-closure manifest
WHEN Mantle prepares rustc, linker, archive, or helper-tool compatibility normalization
THEN every generated wrapper or normalization rule MUST be derived from declared closure members and written as receipt-bound stage material.
AND the proof bundle MUST record each generated file path, BLAKE3 digest, selected input digest, and normalization rule.

#### Scenario: external wrappers fail closed

GIVEN `RUSTC`, PATH, or an explicit proof argument points at a rustc/linker wrapper that is not a declared closure member
WHEN the fixed-point command evaluates preflight
THEN Mantle MUST fail before stage topology execution with an external-wrapper blocker.
AND it MUST NOT claim wrapperless, source-root, or source-built toolchain closure proof success.

#### Scenario: both stages enforce the same policy

GIVEN stage1 and stage2 fixed-point builds are launched
WHEN Mantle constructs their execution environments
THEN both stages MUST use the same closure policy digest, guard policy digest, and command-owned normalization plan digest unless a deterministic policy-mismatch blocker is reported.
AND stage2 MUST NOT inherit undeclared host wrappers from the stage1 launcher environment.

#### Scenario: ambient Nix and rustup remain guarded

GIVEN the fixed-point command is running under a host that has Nix, rustup, Cargo, or profile toolchains on ambient PATH
WHEN any stage attempts to resolve a protected tool
THEN resolution MUST use only declared closure members or command-owned wrapper material.
AND undeclared Nix, rustup, Cargo, or ambient toolchain resolution MUST fail with a deterministic host-tool-leakage blocker before a proof claim.

#### Scenario: summary states the remaining frontier

GIVEN wrapperless source-root fixed-point execution completes, blocks, or mismatches
WHEN Mantle writes the proof summary
THEN the summary MUST identify wrapperless status, source-root closure status, per-stage policy digests, fixed-point binary digests when present, exact blocker class when blocked, and bounded non-claims.
AND a successful stage1==stage2 comparison MUST NOT claim release reproducibility, full Cargo compatibility, compiler correctness, or a zero-seed bootstrap root unless separate evidence exists.

### Requirement: Native Rust topology hardening

r[rust_package_planning.native_topology_hardening] Mantle MUST provide deterministic diagnostics, bounded replay evidence, and positive plus negative coverage for native Rust topology behavior that affects Cargo-free builds.

#### Scenario: topology diagnostics name stable identities

GIVEN native Rust topology planning or execution rejects, blocks, or cannot order a unit
WHEN Mantle reports the diagnostic
THEN the diagnostic MUST include stable native unit identity, package identity, execution role, selected triple, target kind, artifact role, and blocker class when available.
AND it MUST NOT require Cargo unit indices as the only way to understand the failure.

#### Scenario: replay evidence is bounded

GIVEN a native Rust unit fails or blocks during topology execution
WHEN Mantle records replay evidence
THEN the receipt MUST contain the deterministic facts needed to explain or replay the unit boundary.
AND large or source-derived inputs MUST be represented by BLAKE3 digests or bounded summaries unless the full inline value is explicitly required.

#### Scenario: role and metadata mismatches fail before rustc

GIVEN a native Rust unit consumes an artifact with the wrong role, selected triple, source package, rustc metadata, or toolchain policy digest
WHEN Mantle validates the topology artifact graph
THEN validation MUST fail before invoking rustc.
AND the diagnostic MUST name the mismatched field and producer/consumer boundary.

#### Scenario: supported host-unit edges remain covered

GIVEN a Rust workspace uses build-script dependencies, linked metadata, proc macros, selected target features, or source-root host/target splits
WHEN focused native topology fixtures run
THEN Mantle MUST either build the supported topology edge or report an explicit unsupported blocker.
AND it MUST NOT silently fall back to ambient Cargo planning.

### Requirement: Cargo-free fixed-point blocker resolution

r[rust_package_planning.cargo_free_fixed_point_blocker_resolution] Mantle MUST convert Cargo-free fixed-point topology blocked statuses into deterministic, actionable blocker diagnostics and resolve implementation-owned blockers before claiming proof success.

#### Scenario: blocked receipt is classified

GIVEN a Cargo-free fixed-point run emits a topology execution status of blocked
WHEN Mantle records the proof receipt or evidence summary
THEN the summary MUST identify the root blocked unit, package identity, execution role, selected triple, predecessor status, and blocker class when those fields are present in the receipt.
AND missing classifier inputs MUST produce a deterministic diagnostic instead of a generic success or opaque blocked claim.

#### Scenario: implementation-owned blocker advances

GIVEN the classifier identifies a blocker caused by Mantle's native Rust planner or topology executor
WHEN the change is validated
THEN Mantle MUST either fix that blocker and show the next proof frontier advanced, or record a narrower evidence-backed reason why the blocker is not implementation-owned.
AND it MUST NOT mark the fixed-point proof complete from synthetic fixtures alone.

#### Scenario: external blocker remains bounded

GIVEN a proof run is blocked by source-root toolchain material, host kernel capability, disk capacity, or another external prerequisite
WHEN the evidence is reported
THEN Mantle MUST name the external blocker, command, receipt path or bundle, and next action.
AND the report MUST remain narrower than a Nix-free fixed-point success claim.

### Requirement: Native topology validation is stable

r[rust_package_planning.native_topology_validation_stability] Native Rust topology validation SHOULD provide focused commands that pass deterministically under documented host tooling and do not rely on hidden fixture ordering.

#### Scenario: Focused serial rail passes

GIVEN the documented Mantle Rust build environment is available
WHEN the focused native rust-plan serial validation command runs
THEN it MUST pass or report a deterministic first-party blocker
AND the evidence MUST include the exact command output summary.

#### Scenario: Repeated validation has the same result

GIVEN no source files change between runs
WHEN the focused validation command is repeated
THEN it SHOULD produce the same pass/blocker status
AND it MUST NOT depend on leftover fixture state from a previous run.

### Requirement: Native topology fixtures are race-free

r[rust_package_planning.native_topology_race_free_fixtures] Native rust-plan tests MUST isolate or explicitly lock shared fixture state such as rustc wrappers, cargo shims, OUT_DIRs, execution roots, compiler-policy files, and ambient environment probes.

#### Scenario: Parallel fixtures do not share output roots

GIVEN two native topology tests run concurrently
WHEN they create compiler wrappers, cargo shims, or execution output directories
THEN each test MUST use a unique root or a documented lock
AND one test MUST NOT observe another test's generated artifacts.

#### Scenario: Ambient environment tests are subprocessed

GIVEN a negative test needs conflicting environment variables
WHEN it proves planner behavior under those variables
THEN it MUST spawn a child process or use an equivalent isolation boundary
AND it MUST NOT mutate process-global environment in a way that races with other tests.

### Requirement: Native topology parallel evidence is explicit

r[rust_package_planning.native_topology_parallel_evidence] Mantle validation evidence MUST distinguish serial-only, parallel-safe, stress-tested, and remaining-blocked native rust-plan rails.

#### Scenario: Parallel rail is claimed only after proof

GIVEN a summary claims native rust-plan focused tests are parallel-safe
WHEN that summary is written
THEN it MUST cite same-run parallel or repeated-run evidence
AND it MUST NOT generalize from a serial-only run.

#### Scenario: Remaining serial-only test is documented

GIVEN a native rust-plan test still requires serialization
WHEN validation docs are updated
THEN the docs MUST name the shared resource or race risk
AND they MUST identify a next action for removing the serial requirement.

#### Scenario: malformed blocker evidence fails closed

GIVEN a blocked topology receipt is malformed, truncated, or missing required identity fields
WHEN Mantle classifies the blocker
THEN classification MUST fail with a deterministic diagnostic.
AND it MUST NOT fabricate unit identity, role, triple, or proof-success evidence.

### Requirement: Vendor material checksum repair remains fail-closed

r[rust_package_planning.vendor_material_checksum_repair] Cargo-free native Rust planning MUST require vendored registry source material, Cargo checksum metadata, and `Cargo.lock` checksum expectations to agree before a source package is admitted into the native source closure.

#### Scenario: Repaired vendored package matches lock material

GIVEN vendored material for a registry package that appears in `Cargo.lock`
WHEN native registry source planning computes the source-material digest
THEN the computed material MUST match the lockfile/checksum metadata expected by the planner
AND planning MUST admit the package without fabricating or ignoring checksum facts.

#### Scenario: Drift remains a hard blocker

GIVEN vendored material differs from the checksum material declared for a registry package
WHEN native registry source planning validates the package
THEN planning MUST fail closed with a deterministic vendor-checksum diagnostic
AND it MUST NOT continue by using ambient Cargo, network refetching, or unchecked source contents.

### Requirement: Vendor source material drift diagnostics are reviewable

r[rust_package_planning.vendor_source_material_drift_diagnostics] Cargo-free source-material diagnostics MUST identify the affected package, blocker class, planning path, and digest evidence needed to reproduce a vendor checksum mismatch.

#### Scenario: Checksum mismatch explains the blocked package

GIVEN a registry package fails vendored checksum validation
WHEN the Cargo-free blocker classifier summarizes the receipt
THEN the diagnostic MUST name the package identity, blocker class, and nested planning path
AND the diagnostic SHOULD include bounded digest evidence rather than unbounded source listings.

### Requirement: Cargo-free fixed-point frontier is rerun after repair

r[rust_package_planning.cargo_free_fixed_point_frontier_rerun] After repairing a known source-material checksum frontier, Mantle MUST rerun the Cargo-free fixed-point proof or record why the rerun was blocked, and the resulting evidence MUST state success, blocked status, or environmental failure explicitly.

#### Scenario: Fixed-point proof succeeds after repair

GIVEN source material validation no longer blocks stage1 planning
WHEN the Cargo-free fixed-point proof completes
THEN the evidence MUST record stage binary digests, fixed-point status, receipt digests, and the command transcript
AND it MUST state that success applies only to the proven proof mode.

#### Scenario: Frontier moves after repair

GIVEN source material validation no longer reports the repaired package mismatch
WHEN the fixed-point proof fails or blocks on a later frontier
THEN the evidence MUST record the next blocker class and location
AND it MUST NOT claim fixed-point success.

### Requirement: Native Rust planning reports compatibility surface matrix results

r[rust_package_planning.compatibility_surface_matrix] Mantle MUST classify native Rust planning and execution results against a bounded compatibility surface matrix when `rust-plan` is used for Cargo-free evidence. Supported surfaces MAY report `cargo-free-bounded-topology`; unsupported or incomplete surfaces MUST report deterministic `blocked-unsupported-surface` diagnostics and MUST NOT invoke Cargo, ambient caches, network sources, or hidden build orchestration to satisfy the matrix.

#### Scenario: supported matrix surface is cargo-free

GIVEN a matrix fixture uses a Rust surface that Mantle's native planner supports
AND all source, toolchain, feature, host artifact, native-link, and dependency material is declared
WHEN `mantle rust-plan --no-cargo-oracle` plans or executes that fixture
THEN Mantle MAY report `cargo-free-bounded-topology`
AND the receipt MUST bind the surface id, source closure digest, unit graph facts, and non-claims.

#### Scenario: unsupported matrix surface blocks without Cargo fallback

GIVEN a matrix fixture uses a Rust surface outside Mantle's native planning or execution subset
WHEN `mantle rust-plan --no-cargo-oracle` evaluates that fixture
THEN Mantle MUST report `blocked-unsupported-surface` with a stable blocker class
AND it MUST NOT run Cargo or read undeclared Cargo registry, git, target, or package-manager caches to complete the plan.

#### Scenario: native-link surfaces are explicit

GIVEN a Rust package uses `links`, `pkg-config`, `cargo:rustc-link-lib`, `cargo:rustc-link-search`, or native C compilation
WHEN Mantle classifies the native Rust planning surface
THEN the receipt MUST either model the link metadata and declared native inputs explicitly or fail with a native-link blocker
AND it MUST NOT infer host library availability from ambient system paths.
