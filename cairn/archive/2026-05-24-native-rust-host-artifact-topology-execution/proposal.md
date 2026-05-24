# Native Rust host-artifact topology execution

## Summary

Prove that Mantle can execute the bounded host-artifact topology from Mantle-owned native Rust planning facts after native host-unit graph planning is ready.

The slice connects the newly explicit `native_host_unit_graph_planning` receipt to the existing `--execute-host-artifact-topology` rail so supported `custom-build` and `proc-macro` host units are compiled before target consumers, rebound into target execution material, and reported with deterministic receipts without using Cargo as the build orchestrator.

## Motivation

Mantle now derives host-unit graph facts for build scripts and proc macros, but the strongest next evidence is executable: host artifacts planned from native facts should be the material consumed by target `rustc` invocations.

This keeps the Cargo-to-Mantle replacement path monotonic: package/target facts -> unit graph facts -> host-unit graph facts -> executable host-artifact topology.

## Scope

In scope:

- Use ready native host-unit graph facts as the supported source for bounded host-artifact topology execution.
- Execute supported `custom-build` and `proc-macro` host derivation nodes before supported target consumers.
- Bind produced host artifacts into `consumed_host_artifacts`, derivation inputs, and matching `--extern`/metadata surfaces before target `rustc`.
- Preserve deterministic host-artifact topology receipts and fail-closed blockers.
- Add focused positive and negative `rust_plan` CLI tests.

Out of scope:

- General Cargo-compatible scheduling.
- Tests/doctests/examples or non-build modes.
- Broad feature resolution, native-link probing beyond already bounded metadata, or full bootstrap correctness.
- Invoking Cargo to orchestrate host or target execution.
