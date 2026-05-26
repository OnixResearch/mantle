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
