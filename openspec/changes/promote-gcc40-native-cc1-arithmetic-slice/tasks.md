## Phase 1: Evidence target

- [ ] [serial] Inspect the current GCC 4.0 native-boundary receipt and installed `cc1` path to choose the smallest arithmetic/control-flow smoke that can be proven without TinyCC delegation.
- [ ] [depends:evidence-target] Add or update the GCC 4.0 native-cc1 arithmetic receipt schema with bounded input, transcript/proof fields, no-TinyCC-delegation evidence, and partial-only parity effect.

## Phase 2: Implementation

- [ ] [depends:receipt-schema] Update `bootstrap/gcc-4.0.ncl` so the bounded `cc1` arithmetic smoke is served by the promoted native path or fails closed before claiming the slice.
- [ ] [depends:receipt-schema] Wire `bootstrap parity-report` validation for the native-cc1 arithmetic receipt while keeping `gcc.4.0` partial and blocking live-bootstrap/Guix.

## Phase 3: Regression coverage

- [ ] [depends:implementation] Add positive coverage for a matching native-cc1 arithmetic receipt and parity row notes.
- [ ] [depends:implementation] Add negative coverage for TinyCC delegation, missing receipt, unsupported schema, stale marker/transcript digest, and parity overclaiming.

## Phase 4: Verification and archive

- [ ] [depends:tests] Run targeted bootstrap parity tests, CLI parity tests, Nickel/shell shape checks for `bootstrap/gcc-4.0.ncl`, strict OpenSpec validation, and archive after implementation completes.
