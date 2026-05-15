## Why

`gcc.4.0` remains a live-bootstrap and Guix blocker because the current evidence proves a pass1 bridge plus bounded semantics, not native GCC 4.0 compiler correctness. The highest-leverage next slice is the installed `cc1` frontier: today the checked native-boundary receipt explicitly records `cc1-tcc-delegation`, so downstream rows cannot distinguish a native GCC frontend from a TinyCC-backed bridge.

## What Changes

- Promote one bounded native `cc1` correctness slice for GCC 4.0.
- Replace the current `cc1-tcc-delegation` frontier with a machine-checked receipt proving the installed `cc1` handles a narrow arithmetic/control-flow C frontend invocation without delegating object emission to TinyCC.
- Keep `gcc.4.0` partial and blocking live-bootstrap/Guix until broader native compiler and generator correctness are proven.

## Capabilities

### Modified Capabilities
- `bootstrap.gcc.version-ladder`: Adds a bounded native `cc1` arithmetic correctness requirement and fail-closed parity evidence.
- `bootstrap.parity.claim-gating`: Distinguishes native `cc1` slice evidence from full GCC 4.0 completion.

## Impact

- **Files**: `bootstrap/gcc-4.0.ncl`, `bootstrap/evidence/gcc-4.0-native-boundary.json` or a new checked GCC 4.0 native-cc1 receipt, `src/bootstrap_parity.rs`, and targeted tests.
- **APIs**: No public CLI change; parity JSON may include updated evidence notes for `gcc.4.0`.
- **Testing**: Validate Nickel/shell shape, run targeted bootstrap parity tests, build/probe the GCC 4.0 derivation when practical, and run a direct installed-`cc1` smoke for the bounded arithmetic/control-flow slice.
