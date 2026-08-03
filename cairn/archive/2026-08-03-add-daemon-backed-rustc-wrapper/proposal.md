# Proposal: Add a daemon-backed Rust compiler cache adapter

## Why

Mantle-native `rust-plan` execution can call the Rust unit cache directly. Ordinary Cargo developer builds use the `RUSTC_WRAPPER` process boundary instead.

Embedding the store in every wrapper process would duplicate startup work, create database contention, and expose remote credentials to compiler children. Argument-only cache identity would also be unsafe.

Mantle needs an optional thin wrapper and one local daemon. Strong cache use must require an explicit declared-input manifest. Unsupported or undeclared invocations must pass through to the real compiler.

## What Changes

- Add an explicit `mantle rust-cache serve` daemon surface.
- Add a thin `mantle-rustc-wrapper` executable for Cargo's `RUSTC_WRAPPER` protocol.
- Add a bounded local Unix-socket protocol between the wrapper and daemon.
- Require a versioned declared-input manifest before strong cache lookup or publication.
- Run eligible compiler misses under the declared input and effect policy.
- Materialize admitted artifacts into Cargo output paths without FUSE or virtiofs mounts.
- Preserve compiler exit status, stdout, stderr, and Cargo-owned scheduling behavior.
- Bypass unsupported queries, incremental compilation, incomplete manifests, and unsupported output shapes.
- Permit signed remote sharing only after the shared Rust unit action-result change is complete.

## Dependencies

The local adapter depends on `persist-rust-unit-castore-results`. Remote reads and publication depend on `share-rust-unit-action-results`.

## Non-Goals

- Replacing Cargo planning, build-script execution, fingerprints, or scheduling.
- Caching arbitrary ambient wrapper invocations from arguments alone.
- Caching Rust incremental compiler state.
- Enabling the wrapper inside strict self-build, witness, or release proof lanes by default.
- Claiming full Cargo compatibility, compiler correctness, or hermeticity for pass-through invocations.

## Impact

- **Files**: `crates/crunch-rust-cache-core/`, `crates/crunch-rust-cache/`, a wrapper binary, daemon CLI wiring, typed Nickel policy, Cargo adapter tests, documentation, and Cairn evidence.
- **Testing**: protocol bounds, peer authorization, declared-input eligibility, pass-through parity, local hits, remote hits, output restoration, compiler failures, unsupported invocations, and daemon loss.
