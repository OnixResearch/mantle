# Rust Package Planning Delta: Native Unit Graph Planning

## ADDED Requirements

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
