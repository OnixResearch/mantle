# first-stage musl rustc-driver rlib

## Problem

A real source-root musl-host Rust provider rerun now reaches the `run_rustc` stage that builds `compiler/rustc`, but Cargo stops at `rustc_driver` because Rust 1.90 declares `crate-type = ["dylib"]`. The `x86_64-unknown-linux-musl` compiler-host route does not support producing `dylib` crate types there, so the first-stage provider cannot produce the musl-host `rustc` binary.

## Proposed change

Patch the verified Rust source tree inside the generated first-stage script, after mrustc has unpacked/applied its Rust 1.90 source patches and before `run_rustc` builds the compiler host. The patch is musl-host-only: it rewrites `compiler/rustc_driver/Cargo.toml` from `crate-type = ["dylib"]` to `crate-type = ["rlib"]`, accepts an already-patched `rlib` line for idempotent scratch reruns, and fails closed if the expected manifest or crate-type line is absent.

## Success criteria

- The generated first-stage script contains an explicit musl-host guard, exact `rustc_driver` manifest path, `dylib`→`rlib` rewrite, idempotent `rlib` recognition, and fail-closed diagnostics.
- Focused `rust_source_provider` tests prove the generated script carries the guarded source patch without changing generic host aliases.
- Evidence records the real rerun failure at `rustc_driver` and the scratch continuation showing the `rlib` patch clears that blocker and moves the frontier to dynamic proc-macro loading.
