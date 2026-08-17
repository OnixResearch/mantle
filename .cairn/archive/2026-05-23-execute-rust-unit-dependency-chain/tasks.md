## Phase 1: Planning scaffold

- [x] [serial] Define the bounded dependency-chain execution contract. r[rust_package_planning.unit_execution.dependency_chain]
- [x] [serial] Define chain evidence around per-unit execution receipts. r[rust_package_planning.unit_execution_receipts.dependency_chain]
- [x] [serial] Define fail-closed blockers for missing/stale dependency artifacts and unsupported chain shapes. r[rust_package_planning.unit_execution_blockers.dependency_chain]

## Phase 2: Implementation slices

- [x] [serial] Execute one producer `lib` unit and one consuming supported target unit by rewriting declared dependency artifacts from the producer receipt. r[rust_package_planning.unit_execution.dependency_chain]
- [x] [serial] Emit ordered chain evidence while preserving per-unit execution receipt identity and bounded claims. r[rust_package_planning.unit_execution_receipts.dependency_chain]
- [x] [serial] Add negative coverage proving missing/stale dependency artifact material fails before consumer `rustc`. r[rust_package_planning.unit_execution_blockers.dependency_chain]
