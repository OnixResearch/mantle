## Why

The bootstrap parity report currently lists `gcc.10` as a partial live-bootstrap/Guix row with no checked receipt, no row note, and no evidence-specific fail-closed validation. That makes it weaker than nearby partial rows such as `gcc.4.7`, where a contract-only receipt proves the derivation still carries the expected C/C++ provider shape while explicitly not claiming full native correctness.

## What Changes

- Add a checked GCC 10 provider-contract receipt for `bootstrap/gcc-10.ncl`.
- Teach the parity report to validate the receipt against required configure/build/install/smoke markers in the GCC 10 derivation.
- Keep `gcc.10` `partial`, not complete, until native/full GCC 10 correctness and source transcripts exist.
- Add positive and negative regression coverage so missing receipts, malformed receipts, or marker drift fail closed.

## Capabilities

### Modified Capabilities
- `bootstrap.parity.map`: the `gcc.10` row becomes evidence-backed partial instead of an unannotated partial.
- `bootstrap.source.chain.implementation`: GCC 10 transition evidence includes a checked provider-contract receipt before it can support full-source bootstrap claims.

## Impact

- **Files**: `src/bootstrap_parity.rs`, `bootstrap/evidence/gcc-10-provider-contract.json`, tests, and OpenSpec bootstrap delta.
- **APIs**: No CLI schema change; existing `mantle bootstrap parity-report` output gains row notes/evidence failure detail for `gcc.10`.
- **Dependencies**: None.
- **Testing**: Targeted bootstrap parity unit/CLI tests, parity-report JSON, OpenSpec validation, formatting, and whitespace checks.
