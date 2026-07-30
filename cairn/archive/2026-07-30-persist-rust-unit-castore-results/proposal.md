# Proposal: Persist Rust unit results in Mantle castore

## Why

`mantle rust-plan` can reuse a unit only while its prior execution directory and receipt remain available. Removing that directory forces another compiler invocation, even when Mantle still has suitable content-addressed storage.

Mantle needs a local micro-cache for supported Rust units. The cache must reuse the existing castore without treating object presence as action-result authority.

## What Changes

- Add a versioned BLAKE3 Rust unit action identity over all declared execution inputs.
- Add a Rust unit result record that links one action identity to an immutable castore output tree and artifact manifest.
- Keep current execution-directory reuse as the first lookup route.
- Restore admitted castore results through verified temporary materialization and an atomic filesystem commit.
- Ingest successful compiler outputs before publishing an action-result record.
- Add bounded retention, garbage-collection, diagnostics, and performance evidence for Rust unit results.
- Record writable FUSE and virtiofs target-directory restoration as non-goals.

## Dependencies

This change applies ADR 0024 to Rust unit execution. It does not depend on remote publication or a Cargo wrapper.

## Non-Goals

- Full Cargo compatibility.
- Compiler correctness or reproducibility claims.
- Ambient Cargo target-directory reuse.
- Remote cache discovery or publication.
- Writable FUSE or virtiofs mounts over Cargo output directories.

## Impact

- **Files**: `crates/crunch-rust-cache-core/`, `crates/crunch-rust-cache/`, `src/rust_plan.rs`, `src/main.rs`, focused store integration, documentation, and Cairn evidence.
- **Testing**: canonical identity tests, local hit and miss tests, corrupt-result tests, atomic restoration tests, retention tests, and a bounded performance comparison.
