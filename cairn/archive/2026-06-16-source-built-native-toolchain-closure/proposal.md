## Why

The current musl-target fixed-point proof binds target-prefixed tools through the receipt-bound PATH, but it still keeps `not-source-built-toolchain-closure` because the native closure manifest contains seed exceptions for host link tooling and target C/binutils helpers. Rust provider evidence is now available, so Mantle needs a precise promotion rule for explicit full toolchain closure manifests and current evidence for the remaining native blocker.

## What Changes

- Promote an explicitly enforced toolchain closure to a source-built claim only when every member is source-built and the manifest has zero seed exceptions.
- Keep explicit closure manifests authoritative over provider-derived status while preserving `not-source-built-toolchain-closure` for any seed exception or partial source-built accounting.
- Attempt the next no-seed/native closure proof and record the deterministic blocker if host GNU C/libc/linker closure is still missing.

## Impact

- **Files**: `src/source_toolchain_closure.rs`, `src/cargo_free_self_build.rs`, `cairn/specs/rust-package-planning/spec.md`.
- **Testing**: focused positive/negative status tests, source/toolchain closure tests, Cargo-free summary tests, Cairn gates, and a current fixed-point/proof attempt transcript.
