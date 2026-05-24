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
