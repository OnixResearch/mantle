## Why

`rust-plan --execute-topology` is the broad user-facing Cargo-free Rust execution rail for mixed host and target unit graphs. Native host-unit graph planning and native host-artifact topology execution now prove that supported `custom-build` and `proc-macro` host units can be derived from Mantle-owned facts and executed without Cargo orchestration, but the unified topology entry point still accepts only `unit_derivation_graph` material.

That leaves the main topology rail under-specified: a mixed graph could execute host artifacts through derivation evidence without first proving that those host units are backed by ready `native_host_unit_graph_planning` facts. This change closes that seam so the unified topology rail cannot drift back toward Cargo-derived, ambiguous, or non-native host material.

## What Changes

- Require `native_host_unit_graph_planning.ready=true` for mixed host+target `rust-plan --execute-topology` execution.
- Thread native host-unit graph evidence into the unified topology executor and fail closed before host or target `rustc` when host graph material is blocked, missing, non-native, or inconsistent with derivation surfaces.
- Preserve the existing bounded unified topology receipt shape while making its host-material claim explicitly native-planned.
- Add focused positive/negative CLI tests for mixed proc-macro/build-script topology execution and blocked native host graph behavior.

## Impact

- **Files**: `src/main.rs`, `src/rust_plan.rs`, `tests/rust_plan_cli.rs`, `cairn/specs/rust-package-planning/spec.md`.
- **Testing**: focused `rust_plan_cli` tests, `cargo fmt --check`, Cairn validate, proposal/design/tasks gates, and `git diff --check` before archive/commit.
