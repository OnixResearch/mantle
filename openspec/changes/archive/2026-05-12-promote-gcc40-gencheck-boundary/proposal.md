## Why

`gcc.4.0` remains the earliest live-bootstrap/Guix compiler blocker. After tightening `genconstants` and `genflags`, the next small generator seam is `gencheck`: it still emits generic stub-labeled object/header outputs. Tightening this seam reduces ambiguous placeholder debt and adds a deterministic boundary check without claiming native GCC generator correctness.

## What Changes

- Promote the GCC 4.0 `gencheck` object/header seam from generic stub labels to a named disabled-tree-checking boundary.
- Add derivation-local checks that the generated `tree-check.h` boundary contains the guard and marker and rejects the old generic label.
- Refresh GCC 4.0 placeholder inventory and parity regression coverage.

## Capabilities

### Modified Capabilities
- `bootstrap.gcc40.transition`: tightens one more generator boundary while keeping `gcc.4.0` partial.

## Impact

- **Files**: `bootstrap/gcc-4.0.ncl`, GCC evidence receipt, parity tests, bootstrap spec.
- **APIs**: No public CLI change.
- **Testing**: GCC eval shell syntax, GCC build, Rust parity tests, parity-report JSON, OpenSpec validation.
