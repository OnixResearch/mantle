## Why

`gcc.4.0` remains a live-bootstrap/Guix compiler blocker. After tightening `genconstants`, `genflags`, `gencheck`, and `genpreds`, the next bounded generator seam is `genattr`: it still emits a minimal attribute header without a named checked boundary. Naming and checking that seam reduces ambiguous generator debt without claiming native generator correctness.

## What Changes

- Promote the GCC 4.0 `genattr` header seam to a named empty-attribute boundary.
- Add derivation-local checks for the generated `insn-attr.h` guard, marker, and `HAVE_ATTR_enabled` contract.
- Refresh GCC 4.0 placeholder inventory and parity regression coverage.

## Capabilities

### Modified Capabilities
- `bootstrap.gcc40.transition`: tightens one more generator boundary while keeping `gcc.4.0` partial.

## Impact

- **Files**: `bootstrap/gcc-4.0.ncl`, GCC placeholder inventory, parity tests, bootstrap spec.
- **APIs**: No public CLI change.
- **Testing**: GCC eval shell syntax, GCC build, Rust parity tests, parity-report JSON, OpenSpec validation.
