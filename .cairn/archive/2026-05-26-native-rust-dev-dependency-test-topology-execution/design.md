# Design: Native Rust dev-dependency test topology execution

## Current state

Mantle's native Rust planning covers vendored-registry source facts, registry topology execution, host-artifact/build-script/proc-macro rails, transitive registry edges, selected features, target-cfg edges, workspace inheritance, patch/glob surfaces, and dev-dependency test topology planning. The remaining gap is executing a bounded test topology from those native facts without invoking Cargo as orchestrator.

## Approach

- Add or extend a receipt field such as `native_rust_dev_dependency_test_topology_execution` adjacent to existing topology execution receipts.
- Require `native_rust_dev_dependency_test_topology_planning.ready=true` and ready source/topology facts before selecting a dev-dependency test unit for execution.
- Reuse existing explicit topology execution scheduling where possible: dev-dependency producers execute before the test consumer, and produced artifacts are rebound only through declared dependency/input/`--extern` surfaces.
- Keep Cargo metadata/unit-graph material as oracle evidence only; do not use Cargo to plan, execute, discover, or repair the new execution claim.
- Emit deterministic blockers before `rustc` for unsupported modes (`doctest`, `run`, `bench`), missing/stale vendor roots, missing test-unit derivation material, multiple ambiguous test consumers, unsupported harness behavior, or normal-build topology requests that would need dev-dependency material.

## Receipts and blockers

Execution receipts should identify workspace root, package/member identity, test target identity, selected dev-dependency key/package identity, source closure digest, registry checksum/source digest evidence, ordered unit executions, produced artifact digests, consumer artifact bindings, toolchain identity, rustc argument digest, blocker class, and stable receipt hash. Receipt claim text must stay bounded to the explicit test topology fixture and must not claim full Cargo test compatibility.

Blocker classes should be stable strings for CLI assertions, for example `dev-dependency-test-planning-not-ready`, `unsupported-test-mode`, `missing-dev-dependency-artifact`, `missing-dev-dependency-source`, `unsupported-cargo-test-harness`, and `normal-topology-dev-dependency-blocked`.

## Verification

- Add a positive `rust_plan_cli` fixture for a local package test target that consumes a vendored-registry dev-dependency and executes through explicit receipts.
- Add a negative fixture proving unsupported test/doctest/run/bench or missing vendor material blocks before `rustc`.
- Add an assertion that normal build topology execution does not silently consume dev-dependency artifacts.
- Run `cargo fmt --check`, focused `rust_plan`/`rust_plan_cli` tests, `cargo test --bin mantle rust_plan`, `cargo test --test rust_plan_cli`, `git diff --check`, Cairn validate, and Cairn proposal/design/tasks gates.

## Risks

The largest risk is overclaiming Cargo test compatibility. Keep the first seam fixture-shaped, receipt-backed, and explicit about non-goals; unsupported shapes should remain deterministic blockers until separate Cairn packages cover them.
