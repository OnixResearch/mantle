## Why

The archived source-built toolchain closure proof successfully bound the target C toolchain and proved fixed-point behavior, but it intentionally preserved seed exceptions for the Rust compiler, Rust sysroot/std, host pkg-config, and host linker compatibility helpers. The proof still reports `not-source-built-toolchain-closure`. The next trust-reduction frontier is to replace the Rust compiler/sysroot seed exceptions with a receipt-bound, source-built Rust seed closure or to fail closed with a precise blocker.

## What Changes

- Define the source-built Rust compiler/sysroot closure contract that can replace the current Rust seed exceptions.
- Materialize or import a real Rust compiler and standard library closure with source identity, build receipt identity, executable paths, and BLAKE3 content digests.
- Thread that closure into Cargo-free self-build and fixed-point proofs without using ambient Rust toolchain discovery.
- Keep non-claims visible until the Rust compiler/sysroot closure is actually source-built and receipt-bound.

## Impact

- **Files**: bootstrap Rust/toolchain definitions, `src/source_toolchain_closure.rs`, `src/cargo_free_self_build.rs`, proof scripts/evidence, and source-root provider material as needed.
- **Testing**: focused closure validator tests, Cargo-free self-build/fixed-point proofs, source-built Rust smoke tests, `cairn validate --root .`, and tasks gate evidence.
