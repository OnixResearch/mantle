# Proposal: Make native build-script execution practical

## Problem

Many real crates rely on build-script runtime behavior: Cargo-provided environment, `OUT_DIR`, `CARGO_CFG_*`, profile variables, `links` metadata, native link directives, rerun hints, and tool helper crates like `cc-rs`. Mantle supports pieces of this today, but coverage is incremental and not yet a coherent build-script contract.

## Change

Define and implement a bounded build-script runtime parity layer. Build scripts run as explicit host units, produce typed metadata receipts, and expose only declared, deterministic environment/material to target units.

## Impact

- **Files**: build-script execution env, metadata parser, receipt model, tests.
- **Testing**: cc-rs/pkg-config-like fixtures, malformed metadata negative tests, self-probes.
