## Why

`gcc.4.0` remains a live-bootstrap/Guix compiler blocker. After tightening `genattr`, the remaining generated-header forest from `gengtype` still uses generic stub labels. Naming and checking that seam reduces ambiguous generator debt without claiming native GTY traversal correctness.

## What Changes

- Promote the GCC 4.0 `gengtype` generated-header/descriptor seam to named empty-GTY boundaries.
- Add derivation-local checks for a generated GTY header, descriptor source, and legacy-label absence.
- Refresh GCC 4.0 placeholder inventory and parity regression coverage.

## Capabilities

### Modified Capabilities
- `bootstrap.gcc40.transition`: tightens one more generator boundary while keeping `gcc.4.0` partial.

## Impact

- **Files**: `bootstrap/gcc-4.0.ncl`, GCC placeholder inventory, parity tests, bootstrap spec.
- **APIs**: No public CLI change.
- **Testing**: GCC eval shell syntax, GCC build, Rust parity tests, parity-report JSON, OpenSpec validation.
