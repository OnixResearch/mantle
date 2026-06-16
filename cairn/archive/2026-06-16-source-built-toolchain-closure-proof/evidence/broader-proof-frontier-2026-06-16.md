# Broader proof frontier

Task-ID: H1
Covers: rust_package_planning.source_built_toolchain_closure.provider_status

## Decision

The provider-backed fixed-point proof can retire the stale Rust compiler/sysroot closure non-claim once `--rust-source-provider` validates source-built Rust provider metadata and both stages bind the same provider policy digest.

This does **not** retire broader proof bounds:

- not Crunch bootstrap;
- not release reproducibility;
- not full Cargo compatibility;
- not a fully minimized bootstrap trust root;
- not receipt-bound closure for every non-Rust host linker/tool input unless a separate `--toolchain-closure` manifest is supplied and enforced.

## Next action

A later change should either:

1. materialize and validate a full source-built native toolchain closure manifest covering linker, C/C++ compiler, sysroot, crt objects, runtime libraries, pkg-config, and native helpers; or
2. explicitly split the receipt schema into `rust_source_provider_closure` and `native_toolchain_closure` if operators need those claims separately.

This change keeps the explicit `--toolchain-closure` manifest path authoritative when supplied and only promotes the already-validated Rust provider into the existing summary status when that broader manifest is absent.
