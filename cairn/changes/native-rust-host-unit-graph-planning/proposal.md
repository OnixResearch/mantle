## Why

Mantle now computes supported normal Rust lib/bin unit graph facts from Mantle-owned package, target, and source-closure facts. That native fragment still treats build scripts and proc macros as unsupported edges, which leaves Cargo as the only reviewed source for host-unit topology decisions.

The next replacement boundary is to plan host units natively before adding broader scheduling or execution behavior: derive `custom-build` and `proc-macro` host unit facts from Mantle-owned manifests/targets, bind the target units that consume those host artifacts, retain Cargo unit-graph output only as oracle comparison evidence, and fail closed on unsupported or mismatched host surfaces.

## What Changes

- Add a native host-unit graph planning receipt surface under `rust-plan` next to `native_unit_graph_planning`.
- Model bounded `custom-build` and `proc-macro` host units, their host execution kind, expected artifact classes, generated-metadata placeholders, and target-consumer edges from Mantle-owned package/target/source facts.
- Keep Cargo `unit-graph` as oracle/evidence only; native host-unit readiness requires deterministic comparison against the oracle.
- Feed ready native host-unit facts into `unit_derivation_graph` for the supported fragment instead of using Cargo as the source of host/target topology facts.
- Emit deterministic blockers for unsupported host surfaces, unresolved consumer edges, missing native facts, host/target confusion, and native-vs-oracle mismatches.

## Impact

- **Files**: `src/rust_plan.rs`, focused `rust_plan` fixtures/tests, and `cairn/specs/rust-package-planning/spec.md`.
- **Testing**: focused positive and negative `rust_plan` tests plus Cairn validate and proposal/design/tasks gates.
