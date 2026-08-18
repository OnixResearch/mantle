# Proposal: execute-rust-unified-topology

## Summary

Expose one bounded Cargo-free Rust topology execution rail that can run mixed explicit host and target units from Mantle's unit derivation graph.

## Motivation

Mantle currently has separate execution rails for a first supported unit, a single dependency chain, target-only topology, and host-artifact topology. Those seams proved the pieces independently, but users still need to know which rail matches the graph shape. A single bounded topology rail is the next ROI because it can execute normal target dependencies plus proc-macro/build-script host artifacts through one reviewable receipt.

## Scope

- Add a `rust-plan --execute-topology` CLI rail requiring `--execution-output-root`.
- Execute supported host units first, then all supported target units in dependency order.
- Bind produced proc-macro/build-script artifacts, build-script metadata, and target library artifacts before invoking dependent rustc units.
- Emit retained Rust plan evidence plus ordered topology execution receipts and blockers.
- Add positive and negative CLI fixtures.

## Non-goals

- Full Cargo scheduler parity.
- Parallel scheduling.
- Unsupported unit modes or target kinds.
- Native source compilation beyond already-captured build-script metadata.
