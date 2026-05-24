## Phase 1: Specification

- [x] [serial] Define the bounded native package/target planning fragment and Cargo-oracle comparison boundary. r[rust_package_planning.native_package_target_planning]
- [x] [serial] Define deterministic mismatch and unsupported-shape blockers for the native planning fragment. r[rust_package_planning.native_package_target_planning.blockers]
- [x] [serial] Define receipt/CLI evidence for retained Cargo oracle plus Mantle-computed fragment facts. r[rust_package_planning.native_package_target_planning.receipts]

## Phase 2: Implementation

- [ ] [serial] Add Mantle-owned package/target fragment DTOs and deterministic receipt fields to `rust-plan`. r[rust_package_planning.native_package_target_planning.receipts]
- [ ] [serial] Implement simple workspace/package target discovery for supported `lib`/`bin` targets without using Cargo as the source of those facts. r[rust_package_planning.native_package_target_planning]
- [ ] [serial] Compare Mantle-computed package/target facts against Cargo oracle material and emit deterministic mismatch blockers. r[rust_package_planning.native_package_target_planning.compare]
- [ ] [serial] Fail closed for unsupported target kinds, feature/workspace surfaces, missing manifests, unreadable paths, and source/oracle inconsistencies. r[rust_package_planning.native_package_target_planning.blockers]
- [ ] [serial] Add positive supported-workspace tests and negative unsupported/mismatch fixtures. r[rust_package_planning.native_package_target_planning.tests]
- [ ] [serial] Run focused verification, sync accepted specs, archive the change, commit, and push. r[rust_package_planning.native_package_target_planning.verify]
