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
