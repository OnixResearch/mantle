## Why

GCC 4.0 generator wrapper splitting is exhausted, so the next useful bootstrap increment is to make the native-correctness frontier explicit and checked. The existing native-boundary receipt proves the intentional pass1 bridge, but it does not enumerate the remaining non-native seams that keep `gcc.4.0` partial.

## What Changes

- Extend GCC 4.0 parity evidence with a checked native-frontier section.
- Require the receipt to name representative remaining native-correctness blockers from the actual derivation.
- Keep the row evidence-backed `partial`; this is a frontier map, not native GCC correctness.

## Capabilities

### Modified Capabilities
- `bootstrap.parity.claim-gating`: strengthens GCC 4.0 claim gating by making native-frontier debt explicit.

## Impact

- **Files**: `bootstrap/evidence/gcc-4.0-native-boundary.json`, `src/bootstrap_parity.rs`, `openspec/specs/bootstrap/spec.md`.
- **Testing**: parity tests, eval/shell syntax, build, parity report, OpenSpec validation.
