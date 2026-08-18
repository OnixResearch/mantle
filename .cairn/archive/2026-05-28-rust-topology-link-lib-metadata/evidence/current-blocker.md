# Current Blocker

Task-ID: H1
Covers: rust_package_planning.native_link_lib_metadata

## Question

What deterministic frontier remains after native dependency cap-lints parity?

## Inspected evidence

Clean probe receipt from prior completed change:

- Receipt: `target/mantle-self-rust-plan-probe-after-4621b5ba-clean/receipt.json`.
- Topology status: `blocked`.
- Prior frontier moved: `derive_builder_core`, `derive_builder_macro`, and `rustversion` units reported `success`.
- Remaining blocker: `malformed-build-script-metadata: build-script metadata line 75 is malformed: rustc-link-lib name must be a safe token`.

## Decision

Create a bounded parser fix for Cargo-compatible `rustc-link-lib` metadata before claiming the next topology frontier.

## Owner

Mantle agent.

## Next action

Add parser tests, implement fail-closed modifier-aware parsing, and rerun focused validation plus a clean self-probe.
