## Phase 1: Planning

- [x] [serial] Define bounded target-only topology execution requirements and non-goals. r[rust_package_planning.unit_execution.target_topology]
- [x] [serial] Define topology receipt and CLI evidence requirements. r[rust_package_planning.unit_execution_receipts.target_topology.cli]

## Phase 2: Implementation

- [x] [serial] Add deterministic target-only topological execution over explicit unit derivations. r[rust_package_planning.unit_execution.target_topology]
- [x] [serial] Emit topology receipts with ordered unit executions, blockers, BLAKE3 output evidence, and stable receipt hashes. r[rust_package_planning.unit_execution_receipts.target_topology]
- [x] [serial] Expose target topology execution through `rust-plan` CLI JSON evidence. r[rust_package_planning.unit_execution_receipts.target_topology.cli]
- [x] [serial] Add positive multi-crate topology CLI coverage and negative unsupported-shape/blocker coverage. r[rust_package_planning.unit_execution.target_topology.blockers]
