## Why

Mantle's most important next proof still stops at `stage1 blocked: topology execution status was blocked`. That is useful evidence, but it is not yet the Nix-free fixed-point result the operator story needs. The next change should turn that broad blocked status into a deterministic root-cause diagnostic, then resolve any implementation-owned blocker before attempting another fixed-point run.

## What Changes

- Capture the current Cargo-free fixed-point blocker as durable evidence.
- Classify blocked topology receipts into actionable unit/package/role/triple diagnostics.
- Fix the highest-priority implementation-owned blocker in the native Rust topology path.
- Keep any remaining external or source-root blocker explicit instead of weakening the proof claim.

## Impact

- **Files**: `src/cargo_free_self_build.rs`, `src/rust_plan.rs`, proof-runner evidence handling, focused topology tests, Cairn rust-package-planning spec delta.
- **Testing**: blocked-receipt classification, positive blocker-resolution fixture, negative malformed/ambiguous blocker fixture, focused native topology tests, proof rerun or recorded next-blocker evidence, Cairn validation/gates.
