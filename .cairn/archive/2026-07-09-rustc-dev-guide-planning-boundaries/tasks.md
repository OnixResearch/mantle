## Tasks

- [x] [serial] Add a rustc-dev-guide reference map documenting HIR, MIR, backend, rustc_driver, sysroot, crate-type, linker, metadata, and path-remap surfaces. r[rust_package_planning.rustc_dev_guide.reference_map]
- [x] [serial] Add pure validation for claim-bearing guide references, including pinned revision/digest checks and deterministic blockers for moving `main` links. r[rust_package_planning.rustc_dev_guide.reference_map.pinned]
- [x] [serial] Thread guide reference IDs into Rust planning evidence where backend-facing rustc invocation assumptions are claim-bearing. r[rust_package_planning.rustc_dev_guide.backend_invocation]
- [x] [serial] Scope compiler-policy adapter receipts around guide-backed HIR/MIR/rustc_driver boundaries and preserve non-claims for program/compiler correctness. r[rust_package_planning.rustc_dev_guide.compiler_policy_adapter]
- [x] [serial] Bind source-built Rust provider patch-plan operations to guide-backed rustc_driver/sysroot/codegen source anchors where those operations rely on rustc internals. r[rust_package_planning.rustc_dev_guide.source_provider_patch_plan]
- [x] [serial] Add positive fixtures for pinned guide references with bounded Rust planning, compiler-policy, and source-provider claims. r[rust_package_planning.rustc_dev_guide.final_validation]
- [x] [serial] Add negative fixtures for unpinned guide links, missing guide references, unsupported HIR/MIR semantic claims, and unstated backend assumptions. r[rust_package_planning.rustc_dev_guide.final_validation]
- [x] [serial] Run focused rust-plan tests plus Cairn validate/proposal/design/tasks gates before archive. r[rust_package_planning.rustc_dev_guide.final_validation]
