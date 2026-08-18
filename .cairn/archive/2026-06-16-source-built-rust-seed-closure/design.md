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

## Current frontier: GNU compiler host with musl target rustlib

The real mrustc-to-Rust first-stage route now moves past the earlier env leak, local `libproc_macro` workspace discovery, musl CRT search, unwind-symbol, static-musl `*.so` copy, and musl-host `rustc_driver` dylib blockers. The route plan and first-stage build boundary now model two roles:

- `compiler_host_triple`: the triple used for `rustc`, `cargo`, `rustdoc`, proc-macro loading, and compiler-host libraries. The active route uses `x86_64-unknown-linux-gnu` so `rustc_driver` can be built as a dylib-capable compiler-host artifact.
- `target_triple`: the musl target sysroot used for statically linked Mantle outputs and target package units. The first-stage generator builds this as a std-only target rustlib pass and records it separately as `target-rustlib`.

Provider metadata and receipts continue to carry both `host-rustlib` and `target-rustlib` roles, and tests reject satisfying the target role with the host rustlib path. The preserved clean rebuild recorded in `evidence/host-target-split-implementation-2026-06-12.md` shows the translated Rust 1.90 compiler reporting `host: x86_64-unknown-linux-gnu`, so the old musl-host `rustc_driver` blocker moved.

The next observed manual-probe frontier is translated Cargo dependency build environment: `libz-sys` could not find `zlib.h`, and vendored OpenSSL could not find `make`. The generated first-stage script now carries the make directory in `PATH` and exports zlib `CFLAGS`/`CPPFLAGS`, but this still cannot justify a source-built provider claim until final provider materialization, smoke, provider-backed self-build, and fixed-point proof succeed.

## Validation

- Closure validator rejects placeholder or prebuilt Rust provider metadata.
- Source-built Rust smoke builds a trivial Rust program and reports compiler/sysroot digests.
- First-stage env-isolation tests prove inherited Cargo/build-script variables are cleared before mrustc/minicargo/make can run crate build scripts.
- Cargo-free one-shot self-build succeeds or records a deterministic blocker.
- Fixed-point proof succeeds only if stage binary digests and closure policy digests match.
