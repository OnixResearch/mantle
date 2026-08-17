## Phase 1: CLI evidence contract

- [x] [serial] Define CLI exposure for bounded dependency-chain execution evidence. r[rust_package_planning.unit_execution.dependency_chain.cli]
- [x] [serial] Define CLI receipt envelope preserving plan and chain receipt identity. r[rust_package_planning.unit_execution_receipts.dependency_chain.cli]

## Phase 2: Implementation

- [x] [serial] Add the `rust-plan` CLI flag and route it to the existing dependency-chain executor without changing single-unit behavior. r[rust_package_planning.unit_execution.dependency_chain.cli]
- [x] [serial] Emit a combined JSON receipt with ordered producer/consumer chain evidence. r[rust_package_planning.unit_execution_receipts.dependency_chain.cli]
- [x] [serial] Add focused CLI coverage for a two-crate path dependency chain and output artifact digests. r[rust_package_planning.unit_execution_receipts.dependency_chain.cli]
