## Context

The current fixed-point proof uses `rustc-source-root-target-wrapper` around a prebuilt Nix Rust 1.94.1 compiler and records the Rust sysroot/std as seed exceptions. This was intentionally scoped: target linking uses the source-root musl toolchain, but the Rust compiler itself is not yet source-built by Mantle.

## Approach

1. Inventory every Rust seed exception in the latest proof manifest and map it to required source artifacts, build steps, and runtime inputs.
2. Define a normalized Rust compiler/sysroot provider contract that emits `rustc`, `rustdoc` if needed, target std libraries, host support libraries, and metadata with BLAKE3 digests.
3. Build or import the provider through Mantle-controlled derivations with receipt-bound source identity. If a fully source-built provider is not available, fail closed and record the blocker rather than weakening the claim.
4. Update the toolchain closure manifest generation so Rust compiler/sysroot members can be `source-built` only when the provider metadata and receipts are complete.
5. Rerun one-shot and fixed-point Cargo-free proofs. The proof may drop `not-source-built-toolchain-closure` only when no Rust compiler/sysroot seed exception remains and all source-built closure checks pass.

## Risks

- Rust bootstrap can be large and slow; use bounded smoke/proof stages before full fixed-point.
- Host/target split must remain intact: host proc-macro/custom-build units still need host-compatible Rust artifacts.
- Accidentally relabeling prebuilt Rust as source-built would violate proof-before-claim policy.

## Validation

- Closure validator rejects placeholder or prebuilt Rust provider metadata.
- Source-built Rust smoke builds a trivial Rust program and reports compiler/sysroot digests.
- Cargo-free one-shot self-build succeeds or records a deterministic blocker.
- Fixed-point proof succeeds only if stage binary digests and closure policy digests match.
