## Why

Mantle now tracks the rustc-dev-guide as important compiler-architecture reference material, but Rust planning and compiler-policy work still needs a lifecycle contract that says how guide-backed assumptions become bounded Mantle evidence. Without that contract, future work can cite moving `main` links, blur HIR/MIR/backend semantics into Mantle orchestration claims, or hide rustc-driver/codegen assumptions inside implementation notes.

## What Changes

- Add rustc-dev-guide reference binding requirements for Rust package planning work.
- Require pinned guide references when a Mantle claim depends on HIR, MIR, rustc_driver, sysroot, codegen backend, target, crate-type, linker, or metadata behavior.
- Scope compiler-policy adapters and source-built Rust provider patch plans around explicit guide-backed boundaries.
- Add positive and negative fixtures so unpinned links, unsupported compiler semantic claims, and unstated backend assumptions fail closed.

## Impact

- Mantle keeps Rust planning evidence honest: orchestration receipts stay distinct from rustc semantic correctness.
- Future compiler-policy and source-provider work has a reviewable reference map instead of ad hoc guide citations.
- The README `main` links remain discovery links; claim-bearing evidence must pin the referenced rustc-dev-guide revision or digest.
