## Phase 1: Contract and validation

- [x] [serial] Add the full musl/binutils provider-contract receipt with separate musl and binutils marker sets.
- [x] [depends:contract] Wire `bootstrap parity-report` to validate the receipt and keep `full-musl-binutils` evidence-backed partial.

## Phase 2: Regression coverage

- [x] [depends:validation] Add positive unit coverage for valid receipt consumption.
- [x] [depends:validation] Add negative unit coverage for missing receipt and musl/binutils marker drift.
- [x] [depends:validation] Add CLI coverage that `full-musl-binutils` remains a blocker with checked evidence.

## Phase 3: Verification

- [x] [depends:tests] Run targeted Rust parity tests, CLI parity tests, `git diff --check`, and strict OpenSpec validation.
