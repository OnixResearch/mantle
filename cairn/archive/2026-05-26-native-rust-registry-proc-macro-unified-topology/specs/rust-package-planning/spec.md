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
