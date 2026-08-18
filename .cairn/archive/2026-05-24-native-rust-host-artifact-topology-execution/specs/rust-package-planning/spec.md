## ADDED Requirements

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
