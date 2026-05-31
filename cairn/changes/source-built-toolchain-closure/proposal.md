# Proposal: Close the source-built toolchain proof gap

## Why

Mantle now has evidence for the bounded Cargo-free fixed-point command, but that evidence explicitly does not prove a source-built compiler/toolchain closure. The fresh probe in `evidence/baseline-fixed-point-gap.md` shows the current fixed-point command succeeds while using the host rustup `rustc` path and recording `not-source-built-toolchain-closure` as a non-claim.

That is the next proof boundary to make durable: before Mantle claims a stronger bootstrap or release-style compiler provenance property, the Rust compiler, linker, C toolchain, sysroot, and native helper tools used by the Cargo-free topology rail need receipt-bound source provenance or explicit seed exceptions.

## What Changes

- Define a source-built Rust toolchain closure proof contract for Mantle's Cargo-free self-build and fixed-point rails.
- Add a durable audit bundle shape that binds every compiler/toolchain input to source digests, build receipts, binary digests, and explicit seed/trust-root exceptions.
- Require the self-build/fixed-point stages to consume the receipt-bound toolchain rather than ambient host `rustc`, Cargo caches, Nix profiles, or undeclared PATH tools when making the source-built toolchain closure claim.
- Preserve current bounded non-claims until the new proof succeeds; existing Cargo-free fixed-point success remains valid but narrower.

## Impact

- **Files**: `src/cargo_free_self_build.rs`, `src/main.rs`, `src/rust_plan.rs`, toolchain/provenance helper modules, tests, `cairn/specs/rust-package-planning/spec.md` after sync/archive.
- **Testing**: positive source-built-toolchain closure fixture/proof, negative host-tool leakage tests, missing/digest-mismatch seed tests, real fixed-point command using a receipt-bound toolchain, Cairn validation/gates.
- **Known blocker**: a real normalized source-root/seed contract is required before completion; placeholder providers or host-tool stubs are not sufficient proof.
