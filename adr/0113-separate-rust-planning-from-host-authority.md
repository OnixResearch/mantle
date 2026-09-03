# 0113: Separate Rust planning from host authority

- Status: Accepted
- Date: 2026-09-02

## Context

`src/rust_plan.rs` combines package and unit decisions with filesystem reads, Cargo JSON, Cargo and rustc processes, environment access, cache services, execution, receipts, and CLI errors. Its process-backed Cargo oracle also shared a trait with `Path` and `RunError` values.

The accepted Rust-plan behavior is large. A bulk source move would mix an architecture repair with changes to mature Cargo-free, build-script, proc-macro, cache, compiler-policy, and source-built proof behavior.

No existing OnixResearch component owns Mantle's package selection, feature closure, target classification, unit topology, or compiler effect plans. `crunch-rust-cache-core` owns cache identity and admission only.

## Decision

Add two narrow crates:

- `mantle-rust-plan-core` is `no_std + alloc`. It owns bounded structural admission, package and feature closure, host and target classification, unit topology, deterministic unit effects, observation classification, compatibility status, and receipt preimages.
- `mantle-rust-plan` owns application requests, observations, capability errors, and ports for workspace facts, Cargo oracle facts, compiler inspection, unit execution, and Rust cache access.

Move the process-backed Cargo and compiler adapters out of `src/rust_plan.rs`. Keep path, process, Cargo JSON, rustc, cache, filesystem, and rendering details in the root shell and adapter modules.

Delegate accepted compatibility classification, target-kind selection, host-target selection, crate-name normalization, and native unit identity to the extracted core. Keep the mature legacy facade while its accepted behaviors use these core decisions.

## Consequences

The new core can replay planning from supplied facts on host and `wasm32-unknown-unknown`. Application ports no longer expose `RunError`, path, process, vendor cache, raw store, or Snix types.

Unit effects contain declared arguments, environment facts, input identities, expected outputs, and limits. An effect is not evidence that rustc ran. A matching bounded observation is required for an execution receipt.

Golden fixtures preserve Cargo-oracle facts, compatibility outputs, native unit identity, and receipt-preimage identity. The existing CLI and 222-test Rust-plan behavior remain unchanged.

This decision does not prove Cargo equivalence, rustc correctness, linker correctness, cache correctness, full ecosystem support, reproducibility, or release eligibility.
