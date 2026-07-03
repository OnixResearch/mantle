## Why

Native Rust topology is now the center of the Cargo-free proof path. It handles host/target roles, build scripts, proc macros, source-root toolchains, and receipt-bound artifact identity. The next hardening pass should improve diagnostics, replayability, and edge-case coverage without reopening Cargo as the hidden planner.

## What Changes

- Tighten native topology diagnostics for blocked units, missing artifacts, role/triple mismatches, and metadata-search failures.
- Minimize and stabilize receipts needed to replay a failing unit.
- Add edge-case fixtures for selected host units, build-script metadata, proc-macro dependencies, and source-root target/host splits.
- Keep planner logic functional-core testable and shell side effects in execution code.

## Impact

- **Files**: `src/rust_plan.rs`, `src/cargo_free_self_build.rs`, topology receipt structs, focused fixtures/tests, Cairn rust-package-planning spec delta.
- **Testing**: positive host/target topology fixtures, negative role/metadata mismatch fixtures, replay receipt checks, focused cargo-free self-build status tests, Cairn validation/gates.
