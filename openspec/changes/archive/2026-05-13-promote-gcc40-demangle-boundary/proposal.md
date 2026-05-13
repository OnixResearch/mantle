## Why

The native-frontier receipt now names libiberty's `cp-demangle.c` bootstrap shim as one remaining non-native seam. The current marker is a generic stub symbol, so it is indistinguishable from an unexamined placeholder. We can reduce that blocker by converting it into an explicit disabled-demangle boundary with checked semantics while keeping GCC 4.0 partial.

## What Changes

- Replace `libiberty_cp_demangle_bootstrap_stub` with a bounded disabled-demangle source boundary.
- Update native-frontier evidence to reference the new checked marker rather than the generic stub.
- Add parity tests that reject the old marker and require the new boundary marker.

## Capabilities

### Modified Capabilities
- `bootstrap.parity.claim-gating`: tracks a reduced GCC 4.0 native frontier where libiberty demangling is explicit disabled-boundary evidence, not a generic shim marker.

## Impact

- **Files**: `bootstrap/gcc-4.0.ncl`, `bootstrap/evidence/gcc-4.0-native-boundary.json`, `bootstrap/evidence/gcc-4.0-placeholder-inventory.json`, `src/bootstrap_parity.rs`, `openspec/specs/bootstrap/spec.md`.
- **Testing**: eval/shell syntax, parity tests, build, parity report, OpenSpec validation.
