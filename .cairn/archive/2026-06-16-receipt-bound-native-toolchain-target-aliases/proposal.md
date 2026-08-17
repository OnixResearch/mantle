## Why

Provider-backed Rust fixed-point proofs can now retire the stale Rust compiler/sysroot non-claim, but a musl-target proof still blocks when target build scripts look for `x86_64-linux-musl-gcc`. Mantle's receipt-bound PATH currently exposes role aliases such as `cc` and `ld`, but it does not guarantee that target-prefixed tool names from the closure manifest are available when build-script ecosystems derive those names from the target triple.

## What Changes

- Expose each executable toolchain-closure member by its declared member name in addition to its file basename and generic role alias.
- Keep Cargo guarded and fail closed on alias conflicts so ambient PATH cannot satisfy target-prefixed tool lookups.
- Validate the musl target proof path with a receipt-bound closure manifest and document the remaining broader release/bootstrap bounds.

## Impact

- **Files**: `src/cargo_free_self_build.rs`, `cairn/specs/rust-package-planning/spec.md`.
- **Testing**: focused alias unit tests, source/toolchain module tests, Cairn gates, and a provider-backed musl-target proof attempt with a receipt-bound native toolchain closure.
