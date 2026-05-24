## Tasks

- [ ] [serial] r[rust_package_planning.native_host_unit_graph_planning.compare] Add native host-unit graph DTOs/receipt fields that derive `custom-build` and `proc-macro` host units from Mantle-owned package, target, and source-closure facts while retaining Cargo only as oracle evidence.
- [ ] [serial] r[rust_package_planning.native_host_unit_graph_planning.receipts] Compute canonical native host graph, Cargo host oracle, comparison, and self-reference-safe receipt hashes with deterministic ready/blocker semantics.
- [ ] [serial] r[rust_package_planning.native_host_unit_graph_planning.consumes_native] Feed ready native host-unit facts into `unit_derivation_graph` so host nodes and target host-artifact consumer edges come from Mantle-owned facts.
- [ ] [serial] r[rust_package_planning.native_host_unit_graph_planning.blockers] Add fail-closed blockers for unsupported host surfaces, missing native facts, unresolved or ambiguous host/target edges, and host/target confusion.
- [ ] [serial] r[rust_package_planning.native_host_unit_graph_planning.tests] Add positive and negative focused `rust_plan` tests covering ready host-unit planning, deterministic unsupported/missing-edge blockers, and native-vs-oracle mismatch blockers.
- [ ] [serial] r[rust_package_planning.native_host_unit_graph_planning.verify] Run focused Rust verification plus Cairn validate/proposal/design/tasks gates, then sync/archive the change after implementation evidence passes.
