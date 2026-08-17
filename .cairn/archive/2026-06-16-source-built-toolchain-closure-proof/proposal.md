## Why

The source-built Rust provider can now build and smoke a real Rust compiler/sysroot, and provider-backed Cargo-free fixed-point proofs succeed. Mantle still reports those proofs as `not-source-built-toolchain-closure` because fixed-point summaries only consider the older explicit `--toolchain-closure` manifest path. That stale non-claim hides the provider evidence operators need to distinguish a provider-backed Rust closure from an ambient Rust toolchain run.

## What Changes

- Promote a validated `--rust-source-provider` into the fixed-point `source_built_toolchain_closure` summary when no explicit closure manifest is supplied.
- Remove `not-source-built-toolchain-closure` from one-shot and fixed-point non-claims when the Rust provider supplies the validated source-built compiler/sysroot closure evidence.
- Keep fail-closed behavior for absent providers, prebuilt provider metadata, invalid metadata, mismatched stage policy digests, and broader non-claims such as release reproducibility and full Cargo compatibility.

## Impact

- **Files**: `src/source_toolchain_closure.rs`, `src/cargo_free_self_build.rs`, `cairn/specs/rust-package-planning/spec.md`.
- **Testing**: focused unit tests for positive provider-backed claim promotion and negative absent/prebuilt/manifest mismatch paths, then provider-backed fixed-point proof evidence.
