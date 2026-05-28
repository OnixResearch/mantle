## Phase 1: Implementation

- [ ] [serial] Record current clean self-probe blocker evidence for Rust 2024 let-chain edition mismatch. Evidence: `evidence/current-blocker.md`. r[rust_package_planning.native_manifest_edition_derivations]
- [ ] [serial] Add focused positive and negative tests for declared edition propagation and missing-edition default behavior. Evidence: `evidence/verification.md`, focused rust-plan tests. r[rust_package_planning.native_manifest_edition_derivations]
- [ ] [serial] Thread package edition through native target facts, native unit facts, native host unit facts, and generated rustc args. Evidence: `src/rust_plan.rs`. r[rust_package_planning.native_manifest_edition_derivations]
- [ ] [serial] Run focused tests, self-probe, Cairn validation, and sync/archive readiness. Evidence: `evidence/verification.md`. r[rust_package_planning.native_manifest_edition_derivations]
