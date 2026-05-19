## Why

`gcc.4.0` remains a live-bootstrap and Guix blocker after the bounded native `cc1` arithmetic/control-flow slice because the native frontier receipt still records checked empty generator boundaries. Those boundaries are useful fail-closed evidence, but they are not native GCC generator correctness and should not satisfy full-source parity.

The next small, high-ROI slice is to promote one generator frontier member from an empty-boundary shim to a checked bounded native output contract. `genattrtab` is a good first target because the current native frontier explicitly names the empty attrtab boundary and it is late enough to exercise the generator-output surface without claiming the whole generator family.

## What Changes

- Promote one bounded GCC 4.0 generator frontier slice for `genattrtab`.
- Replace or narrow the current `genattrtab` empty-boundary marker with a checked receipt proving the bounded output contract for the selected generator case.
- Keep `gcc.4.0` evidence-backed `partial`; this must not claim full native GCC 4.0 compiler, generator-family, live-bootstrap, or Guix parity completion.

## Capabilities

### Modified Capabilities
- `bootstrap.gcc.version-ladder`: Adds a bounded native generator-frontier slice requirement for GCC 4.0 `genattrtab`.
- `bootstrap.parity.claim-gating`: Ensures generator-slice evidence strengthens the partial row while still blocking full parity until broader native compiler/generator correctness exists.

## Impact

- **Files**: likely `bootstrap/gcc-4.0.ncl`, `bootstrap/evidence/gcc-4.0-native-boundary.json`, a new checked generator receipt under `bootstrap/evidence/`, `src/bootstrap_parity.rs`, and targeted tests.
- **APIs**: No public CLI change; parity JSON may include updated `gcc.4.0` evidence notes.
- **Testing**: OpenSpec strict validation, targeted bootstrap parity tests, receipt drift/negative tests, and the parity snapshot rail.
