## Why

`gcc.4.0` remains a live-bootstrap and Guix blocker after the bounded native `genattrtab` generator slice. The native frontier receipt now records `genoutput` as the remaining checked empty-output generator boundary, which is useful fail-closed evidence but is not native generator correctness.

The next small, high-ROI slice is to promote `genoutput` from an empty-output boundary shim to a checked bounded native-generator output contract. This continues the same narrow receipt-and-parity pattern as the completed `genattrtab` slice without claiming full GCC 4.0 compiler or generator-family correctness.

## What Changes

- Promote one bounded GCC 4.0 generator frontier slice for `genoutput`.
- Replace or narrow the current `genoutput` empty-output boundary marker with a checked receipt proving the bounded output contract for the selected generator case.
- Update the native frontier receipt so remaining generator debt is explicit after `genoutput` promotion.
- Keep `gcc.4.0` evidence-backed `partial`; this must not claim full native GCC 4.0 compiler, generator-family, live-bootstrap, or Guix parity completion.

## Capabilities

### Modified Capabilities
- `bootstrap.gcc.version-ladder`: Extends the bounded native generator-frontier slice requirement to GCC 4.0 `genoutput`.
- `bootstrap.parity.claim-gating`: Ensures generator-slice evidence strengthens the partial row while still blocking full parity until broader native compiler/generator correctness exists.

## Impact

- **Files**: likely `bootstrap/gcc-4.0.ncl`, `bootstrap/evidence/gcc-4.0-native-boundary.json`, `bootstrap/evidence/gcc-4.0-native-generator-slice.json` or a successor receipt, `src/bootstrap_parity.rs`, and targeted tests.
- **APIs**: No public CLI change; parity JSON may include updated `gcc.4.0` evidence notes.
- **Testing**: OpenSpec strict validation, targeted bootstrap parity tests, receipt drift/negative tests, and the parity snapshot rail.
