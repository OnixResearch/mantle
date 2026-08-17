## Why

Mantle's next Nix-beating proof needs Cargo-free Rust topology execution to stop mixing target-built dependency artifacts into host build-script units. Source-root musl runs can build simple Rust links, but host custom-build units still need host-compiled support crates while target libraries keep the requested target triple. Without an explicit role/triple split, the proof can fail with mismatched crate metadata or accidentally accept ambient host artifacts.

## What Changes

- Split native Rust topology units into receipt-bound host, target, and host-dependency roles.
- Make artifact identity include role, selected triple, features, source digest, and toolchain-closure policy so host and target variants cannot collide.
- Route build-script dependencies through the host toolchain/sysroot while preserving target toolchain routing for target libraries and binaries.
- Fail closed before `rustc` when a unit consumes an artifact built for the wrong role, triple, source digest, or toolchain policy.
- Record role/triple/toolchain evidence in topology receipts and blocker summaries.

## Impact

- **Files**: `src/rust_plan.rs`, Rust topology receipt models, `tests/rust_plan_cli.rs`, source-root proof helpers, Cairn rust-package-planning spec delta.
- **Testing**: positive host-dependency build-script topology fixture, positive target-library fixture, negative role/triple mismatch fixture, source-root proof blocker rerun, Cairn validation/gates.
