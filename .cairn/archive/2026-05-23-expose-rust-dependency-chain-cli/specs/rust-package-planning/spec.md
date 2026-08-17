# Rust Package Planning Specification Delta

## ADDED Requirements

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
