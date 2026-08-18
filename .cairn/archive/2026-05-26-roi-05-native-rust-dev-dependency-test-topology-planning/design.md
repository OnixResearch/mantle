# Design: Native Rust dev-dependency test topology planning

## Current state

Mantle's native Rust planning now covers vendored-registry source facts, target topology execution, host-artifact/build-script/proc-macro rails, transitive registry edges, bounded selected features, target-cfg dependency edges, and basic workspace dependency inheritance. This change extends the next ROI surface without reintroducing hidden Cargo orchestration.

## Approach

- Add explicit native receipt fields or extend existing fields for `native_rust_dev_dependency_test_topology_planning` evidence.
- Keep Cargo oracle material as comparison/evidence only; do not use Cargo as planner or executor for the new claim.
- Require ready source/topology facts before any execution claim.
- Reuse existing topology scheduling and artifact digest binding where the new surface resolves to an already-supported package/unit edge.
- Emit deterministic blockers before `rustc` when the manifest shape would need unsupported Cargo behavior.

## Receipts and blockers

Receipts should identify workspace root, package/member identity, dependency key or target/test surface, selected source facts, resolved manifest path when applicable, decision, blocker class, and relevant BLAKE3/source digest evidence. Blockers must be stable strings suitable for CLI assertions.

## Verification

- Add focused positive CLI or unit fixture for `rust_package_planning.native_rust_dev_dependency_test_topology_planning`.
- Add focused negative fixture proving unsupported behavior blocks before `rustc`.
- Run `cargo fmt --check`, focused Rust tests, `cargo test --bin mantle rust_plan`, `cargo test --test rust_plan_cli`, `git diff --check`, `cairn validate --root .`, and Cairn proposal/design/tasks gates.

## Risks

The primary risk is accidentally claiming broader Cargo compatibility. The implementation should stay bounded, name unsupported surfaces explicitly, and keep non-goals out of receipts and final claims.
