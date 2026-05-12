## Why

`gcc.4.0` remains the earliest shared live-bootstrap/Guix compiler blocker. Recent work promoted bounded driver and `cc1` wrapper semantics, but the native GCC build boundary is still represented by a broad `make -C gcc ... || install pass1 bridge` handoff. Without a checked receipt, future work can move or regress the native boundary without the parity report noticing.

## What Changes

- Add a checked GCC 4.0 native-boundary receipt that records the current native build frontier.
- Require the parity report to validate that receipt alongside the existing GCC 4.0 placeholder inventory.
- Add regression coverage for matching, missing, and drifted boundary evidence.

## Capabilities

### Modified Capabilities
- `bootstrap.gcc40.transition`: records exact native-boundary evidence before another GCC 4.0 correctness promotion.

## Impact

- **Files**: `bootstrap/evidence/*`, `src/bootstrap_parity.rs`, `openspec/specs/bootstrap/spec.md`.
- **APIs**: No public CLI change.
- **Testing**: Rust parity tests, parity-report JSON, OpenSpec validation, shell/eval shape checks.
