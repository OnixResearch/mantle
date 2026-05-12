## Why

`gcc.4.0` is still reported as `placeholder` because the derivation contains many intentional pass1 bridge/stub markers. That hides whether the remaining markers are known bounded bridge debt or newly introduced accidental placeholders.

## What Changes

- Add a checked GCC 4.0 placeholder-marker inventory receipt.
- Require the parity report to validate the exact current standalone marker set before treating `gcc.4.0` as evidence-backed partial.
- Keep `gcc.4.0` blocking live-bootstrap/Guix until native compiler correctness is proven.

## Impact

- Files: `src/bootstrap_parity.rs`, `bootstrap/evidence/gcc-4.0-placeholder-inventory.json`, bootstrap OpenSpec baseline.
- Testing: targeted parity unit tests, parity-report JSON, OpenSpec validation, whitespace checks.
