## ADDED Requirements

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
