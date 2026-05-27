## Why

Mantle's native `rust-plan --execute-topology` now reaches ready source, package, unit, host, and derivation planning, but unified topology execution still stops when a host unit depends on a proc-macro host package. Current evidence shows `generator`, `indoc`, and `wasm-bindgen` host units consume `rustversion@1.0.22`, while `rustversion@1.0.22` has an explicit native proc-macro host producer. The topology scheduler only looks for target `lib` producers for host-unit dependency artifacts, so it reports `missing-host-dependency-producer` instead of scheduling the host producer first.

## What Changes

- Teach unified Rust topology execution to recognize selected host-unit dependency artifacts that are produced by supported proc-macro host units.
- Schedule proc-macro host producers before host consumers that use them as `--extern` inputs.
- Keep fail-closed behavior for host dependencies that have neither a target `lib` producer nor a supported proc-macro host producer.
- Preserve source-closure-bounded execution: no Cargo orchestration, no ambient registry cache, no network, no target-dir fallback.

## Impact

- **Files**: `src/rust_plan.rs`, `cairn/specs/rust-package-planning/spec.md`, archived change evidence/tasks.
- **Testing**: focused host-dependency topology tests, self `rust-plan --execute-topology` probe, `cairn validate`, `git diff --check`.
