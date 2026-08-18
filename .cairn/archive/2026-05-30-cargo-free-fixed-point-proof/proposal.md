# Proposal: Prove Cargo-free Mantle fixed point

## Why

Mantle now has a Cargo-free topology self-build proof where a host-built Mantle binary builds a Mantle CLI without invoking Cargo. The next proof rung is stricter: the produced stage1 Mantle binary must build stage2 Mantle through the same Cargo-free native topology rail, and stage1/stage2 binary digests must match.

## What Changes

- Add a Rust proof runner for stage1 -> stage2 Mantle fixed-point comparison.
- Record durable evidence for both stages: receipts, stderr, status, smoke output, Cargo guard marker state, produced binary paths, and BLAKE3 digests.
- Fail closed when either stage blocks, Cargo is invoked, the produced Mantle binary cannot be identified, smoke fails, or stage1/stage2 digests differ.
- State non-claims explicitly: this is not Crunch bootstrap, release reproducibility, source-built compiler closure, or full Cargo compatibility.

## Impact

- **Files**: `scripts/prove-cargo-free-fixed-point.rs`, `cairn/specs/rust-package-planning/spec.md`.
- **Testing**: script preflight, fixed-point proof run or deterministic mismatch/blocker bundle, focused Cairn validation.
