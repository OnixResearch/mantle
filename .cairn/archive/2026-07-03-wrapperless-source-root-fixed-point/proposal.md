## Why

Mantle's Cargo-free fixed-point proof should be runnable by an operator without a hand-written Nix/rustup rustc wrapper. A wrapper workaround can demonstrate progress, but it is not a product surface and it weakens the source-root story. The command needs to own toolchain compatibility normalization, bind it into receipts, and fail closed when it cannot do so from declared source-root inputs.

## What Changes

- Extend the Cargo-free fixed-point command to prepare stage-local rustc/linker normalization from a declared source-root provider or toolchain-closure manifest.
- Record every normalization rule, generated wrapper path, input digest, and selected toolchain member in the proof bundle.
- Guard Cargo, Nix, rustup, and ambient PATH/toolchain leakage in both stages.
- Reject caller-provided untracked rustc wrappers as proof inputs unless they are declared closure members with digest-bound provenance.
- Emit a blocker-first proof summary when command-owned normalization cannot satisfy the selected host/target route.

## Impact

- **Files**: `src/self_build.rs`, `src/rust_plan.rs`, proof-bundle summary models, source-root provider/closure validation, CLI tests, Cairn rust-package-planning spec delta.
- **Testing**: wrapperless positive fixed-point or deterministic blocker bundle, negative ambient wrapper rejection, negative missing normalization member, Cargo/Nix/rustup guard checks, Cairn validation/gates.
