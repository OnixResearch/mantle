# Rust Package Planning Specification Delta

## ADDED Requirements

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
