## Phase 1: Specification

- [x] [serial] Define the bounded native unit graph planning fragment and the Cargo unit-graph oracle comparison boundary. r[rust_package_planning.native_unit_graph_planning]
- [x] [serial] Define deterministic blockers for unsupported native unit graph inputs and native-vs-oracle mismatches. r[rust_package_planning.native_unit_graph_planning.blockers]
- [x] [serial] Define receipt evidence for native unit graph facts, oracle comparison, and downstream derivation consumption. r[rust_package_planning.native_unit_graph_planning.receipts]

## Phase 2: Implementation

- [x] [serial] Add Mantle-owned native unit graph DTOs and deterministic receipt fields to `rust-plan`. r[rust_package_planning.native_unit_graph_planning.receipts]
- [x] [serial] Build native `lib`/`bin` unit nodes and path-dependency edges from native package/target facts without using Cargo unit graph as the source. r[rust_package_planning.native_unit_graph_planning]
- [x] [serial] Compare native unit graph facts against Cargo unit-graph oracle material and emit deterministic mismatch blockers. r[rust_package_planning.native_unit_graph_planning.compare]
- [x] [serial] Feed supported ready native unit graph facts into `unit_derivation_graph` while keeping Cargo unit graph as oracle evidence only. r[rust_package_planning.native_unit_graph_planning.consumes_native]
- [x] [serial] Fail closed for unsupported target kinds, unit modes, feature surfaces, missing native package facts, missing source material, and ambiguous dependency edges. r[rust_package_planning.native_unit_graph_planning.blockers]
- [x] [serial] Add positive supported-workspace tests and negative unsupported/mismatch/missing-edge fixtures. r[rust_package_planning.native_unit_graph_planning.tests]
- [x] [serial] Run focused verification, sync accepted specs, archive the change, commit, and push. r[rust_package_planning.native_unit_graph_planning.verify]
