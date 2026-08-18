# Native Rust registry proc-macro unified topology

## Summary

Extend the bounded native Rust registry unified topology proof so vendored registry-backed `proc-macro` host producers execute through the main `rust-plan --execute-topology` rail from explicit Mantle-owned facts.

The previous registry unified-host slice proved registry-backed `custom-build` host producers in unified topology. This change closes the sibling host-artifact parity gap for supported registry-backed proc-macro crates: ready registry source facts and ready native host-unit graph facts must bind the proc-macro producer, the produced host artifact, and target consumers without Cargo orchestration or ambient registry cache fallback.

## Motivation

Real registry crates often expose proc-macro packages as host artifacts. Mantle already has local/path proc-macro topology coverage and registry build-script unified-host coverage, but it needs explicit receipt evidence that vendored registry proc-macro packages participate in the same native registry source/host graph/topology rails.

## Scope

- Add a bounded vendored-registry proc-macro fixture for `rust-plan --execute-topology`.
- Require ready `native_registry_source_planning`, `native_host_unit_graph_planning`, and `unit_derivation_graph` facts before registry proc-macro execution.
- Execute the registry proc-macro host producer before target consumers.
- Bind produced proc-macro host artifact digests into target consumer receipts and `rustc` material.
- Emit deterministic blockers before `rustc` for missing/stale/unsupported registry proc-macro source or host facts.

## Non-goals

- No Cargo orchestration.
- No network/index fetch.
- No `$CARGO_HOME` or ambient registry cache fallback.
- No version solving.
- No broad Cargo registry compatibility claim.
- No unsupported proc-macro expansion semantics beyond the bounded fixture/proof surface.
