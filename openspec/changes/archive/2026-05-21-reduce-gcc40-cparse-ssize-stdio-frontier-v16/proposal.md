## Why

The v15 GCC 4.0 native `cc1` c-parse frontier showed that `config.h + <stdio.h>` fails, `config-undef6.h + <stdio.h>` succeeds, and the only single-undef passing probe is `ssize_t`.  The next smallest useful slice is to prove whether the failure is specifically the generated `ssize_t` definition being visible before `<stdio.h>`, rather than the remaining five config rewrites or `stdio.h` itself.

## What Changes

- Add compact diagnostic probes around the `ssize_t` definition/order seam for `config.h + <stdio.h>`.
- Record v16 source-frontier evidence that preserves v15 and identifies the bounded `ssize_t` interaction.
- Update parity validation/specs so stale v15 evidence fails closed and `gcc.4.0` remains partial/non-promoting.

## Out of Scope

- Proving native GCC 4.0 `cc1` or full compiler correctness.
- Repairing the production GCC build beyond recording the diagnostic frontier.
- Expanding the active diagnostic derivation past the compact c-parse frontier probes.

## Verification

- Evaluate the diagnostic derivation and keep the builder script below the known argv/env ceiling.
- Validate the updated evidence via focused bootstrap parity tests and `mantle --json bootstrap parity-report`.
- Run source-pin/blocker self-tests, `openspec validate --all --strict`, and `git diff --check`.
