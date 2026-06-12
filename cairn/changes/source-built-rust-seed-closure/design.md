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

## Current frontier: musl-host Rust compiler dylib

The real mrustc-to-Rust first-stage route now moves past the earlier env leak, local `libproc_macro` workspace discovery, musl CRT search, unwind-symbol, and static-musl `*.so` copy blockers. The preserved probe reaches the Rust 1.90 compiler build and fails closed because `rustc_driver` still requests a dylib, while the current route is building host compiler artifacts for `x86_64-unknown-linux-musl`, whose target does not support that crate type.

The next design decision is the host/target split for the source-built provider route: keep the target sysroot on musl for static outputs, but avoid pretending the Rust compiler host itself can be produced as a musl dylib-based compiler without a real upstream-compatible strategy. This still cannot justify a source-built provider claim until final provider materialization, smoke, provider-backed self-build, and fixed-point proof succeed.

## Planned route split

The next implementation slice should make the route plan and first-stage build boundary explicit about two roles:

- `compiler_host_triple`: the triple used for `rustc`, `cargo`, `rustdoc`, proc-macro loading, and compiler-host libraries. The current probe indicates this must not be `x86_64-unknown-linux-musl` unless there is a proven dylib-capable strategy for `rustc_driver`.
- `target_triple`: the musl target sysroot used for statically linked Mantle outputs and target package units.

Provider metadata and receipts must continue to carry both `host-rustlib` and `target-rustlib` roles. If the route temporarily uses a GNU compiler host to get past `rustc_driver`, the evidence must still prove that the musl target sysroot is source-built and receipt-bound. It is not acceptable to collapse the roles or relabel a GNU-host compiler plus a prebuilt musl std as a full source-built Rust closure.

## Validation

- Closure validator rejects placeholder or prebuilt Rust provider metadata.
- Source-built Rust smoke builds a trivial Rust program and reports compiler/sysroot digests.
- First-stage env-isolation tests prove inherited Cargo/build-script variables are cleared before mrustc/minicargo/make can run crate build scripts.
- Cargo-free one-shot self-build succeeds or records a deterministic blocker.
- Fixed-point proof succeeds only if stage binary digests and closure policy digests match.
