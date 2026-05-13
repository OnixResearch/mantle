## Why

`gcc.4.0` remains a live-bootstrap/Guix compiler blocker. After tightening `genconstants`, `genflags`, and `gencheck`, the next bounded generator seam is `genpreds`: it still emits generic stub-labeled predicate header/source outputs. Naming and checking that boundary reduces ambiguous placeholder debt without claiming native generator correctness.

## What Changes

- Promote the GCC 4.0 `genpreds` header/source seam from generic stub labels to a named empty-predicate boundary.
- Add derivation-local checks for both `genpreds -h` and source output.
- Refresh GCC 4.0 placeholder inventory and parity regression coverage.

## Capabilities

### Modified Capabilities
- `bootstrap.gcc40.transition`: tightens one more generator boundary while keeping `gcc.4.0` partial.

## Impact

- **Files**: `bootstrap/gcc-4.0.ncl`, GCC evidence receipt, parity tests, bootstrap spec.
- **APIs**: No public CLI change.
- **Testing**: GCC eval shell syntax, GCC build, Rust parity tests, parity-report JSON, OpenSpec validation.
