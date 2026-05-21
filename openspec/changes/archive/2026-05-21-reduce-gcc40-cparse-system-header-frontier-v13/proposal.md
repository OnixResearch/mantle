## Why

The v12 GCC 4.0 native `cc1` c-parse source frontier narrowed the first failing source include prefix to `config.h` plus `system.h`. `config.h` alone succeeds, while adding `system.h` reproduces the same truncated include-flood shape. The next useful slice should inspect inside `system.h` instead of adding more make-log scraping.

## What Changes

- Add a compact diagnostic probe that records the ordered direct `gcc/system.h` includes.
- Add bounded nested-prefix probes that include normalized `config.h` plus progressively larger direct `system.h` include prefixes.
- Record the new nested `system.h` frontier in the checked c-parse source-frontier receipt.
- Keep `gcc.4.0` partial and explicitly avoid native compiler/source-build correctness claims.

## Capabilities

### Modified Capabilities
- `bootstrap`: GCC 4.0 native c-parse frontier evidence gains a nested `system.h` source-boundary slice.

## Impact

- **Files**: `bootstrap/diag-gcc40-c-parse-boundary.ncl`, `bootstrap/evidence/gcc-4.0-native-cc1-build-frontier.json`, `src/bootstrap_parity.rs`, `openspec/specs/bootstrap/spec.md`.
- **APIs**: none.
- **Dependencies**: none.
- **Testing**: JSON validation, focused bootstrap parity tests, CLI parity tests, blocker/source-pin self-tests, parity-report check, OpenSpec strict validation, and whitespace check.
