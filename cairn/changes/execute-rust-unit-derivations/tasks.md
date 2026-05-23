## Phase 1: Planning scaffold

- [x] [serial] Define the supported Rust unit execution contract. r[rust_package_planning.unit_execution]
- [x] [serial] Define per-unit execution receipt and cache/rebuild evidence. r[rust_package_planning.unit_execution_receipts]
- [x] [serial] Define fail-closed execution blockers for missing sources, dependency artifacts, host artifacts, and declared outputs. r[rust_package_planning.unit_execution_blockers]

## Phase 2: Implementation slices

- [x] [serial] Implement execution of one ready supported `lib`/`bin` unit from `unit_derivation_graph` using explicit `rustc` args/env/inputs. r[rust_package_planning.unit_execution.supported_unit]
- [ ] [serial] Emit deterministic per-unit execution receipts with output artifact digests and rebuild/reuse reason. r[rust_package_planning.unit_execution_receipts.output_identity]
- [ ] [serial] Add negative fixtures proving missing source-closure, dependency artifact, host artifact, or declared-output material fails closed before execution. r[rust_package_planning.unit_execution_blockers.missing_material]
- [ ] [serial] Add focused CLI or integration coverage demonstrating the bounded Cargo-free execution claim for the supported explicit-unit subset. r[rust_package_planning.unit_execution.bounded_claim]
