# Proposal: bind-rust-build-script-link-metadata

## Summary

Bind already-captured build-script native link metadata into Mantle's Cargo-free Rust target execution rail.

## Motivation

Mantle now executes explicit `custom-build` host units and records `cargo:rustc-link-lib` / `cargo:rustc-link-search` evidence, but downstream target execution only consumes `OUT_DIR`, `rustc-env`, and `rustc-cfg`. Crates that require build-script supplied native link arguments still fail even when the metadata has been captured.

## Scope

- Bind supported `rustc-link-search` metadata into deterministic `-L` rustc arguments.
- Bind supported `rustc-link-lib` metadata into deterministic `-l` rustc arguments.
- Fail closed for malformed or unsupported native link metadata before invoking target rustc.
- Add positive and negative CLI fixtures that exercise the host-artifact topology rail.

## Non-goals

- Full Cargo build-script compatibility.
- Native library compilation orchestration beyond explicit build-script behavior.
- Platform-specific framework/linker behavior beyond accepted rustc argument surfaces.
