## Why

`gcc.4.0` remains a live-bootstrap and Guix blocker after the bounded native `cc1`, `genattrtab`, and `genoutput` slices. The current native frontier receipt now leaves libiberty demangling as the named remaining GCC 4.0 native-frontier blocker: it has a checked Itanium zero-argument function slice, but full native `cp-demangle` correctness remains pending.

The next small, high-ROI slice is to extend the bounded demangler from a single flat zero-argument form (`_Z3foov -> foo()`) to one additional deterministic Itanium nested-name zero-argument form. This advances real libiberty semantics while preserving the non-claim that GCC 4.0 is still partial and blocking.

## What Changes

- Add a bounded GCC 4.0 libiberty demangle semantic slice for one nested zero-argument Itanium function shape, for example `_ZN3foo3barEv -> foo::bar()`.
- Record checked receipt or frontier metadata proving the selected input/output contract, source markers, and smoke transcript/digest evidence.
- Update parity validation to require the new bounded demangle evidence, fail closed on stale marker/digest/schema/contract drift, and keep `gcc.4.0` evidence-backed `partial`.
- Do not claim full C++ demangler correctness, broader Itanium ABI coverage, native GCC 4.0 compiler correctness, live-bootstrap parity, or Guix parity.

## Capabilities

### Modified Capabilities
- `bootstrap.gcc.version-ladder`: Extends the GCC 4.0 bounded native demangle semantics requirement with one additional nested-name slice.
- `bootstrap.parity.claim-gating`: Ensures the extra demangle evidence strengthens the partial row while still blocking full parity until broader native compiler/demangler correctness exists.

## Impact

- **Files**: likely `bootstrap/gcc-4.0.ncl`, `bootstrap/evidence/gcc-4.0-native-boundary.json`, a new or updated demangle evidence receipt under `bootstrap/evidence/`, `src/bootstrap_parity.rs`, targeted tests, and synced OpenSpec bootstrap spec.
- **APIs**: No public CLI change; parity JSON may include updated `gcc.4.0` evidence notes.
- **Testing**: OpenSpec strict validation, targeted `bootstrap_parity::tests::gcc40`, receipt drift/negative tests, and the parity snapshot rail.
