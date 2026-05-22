## Why

v18 proved that making the host `<sys/types.h>` `ssize_t` typedef visible before generated `config.h` makes the compact `config.h + <stdio.h>` probe pass. The next useful frontier is to test a production-shaped generated-config adjustment against the real `c-parse.o` make target rather than only synthetic include probes.

## What Changes

- Add one compact diagnostic make probe that appends `#undef ssize_t` to `gcc/config.h` before rebuilding `c-parse.o`, then restores the original config for the unchanged baseline frontier.
- Record the new adjusted-config make frontier in the source-frontier receipt and fail-closed parity checks.
- Keep `gcc.4.0` partial; this is still diagnostic source-frontier evidence, not native GCC correctness.

## Impact

- Files: `bootstrap/diag-gcc40-c-parse-boundary.ncl`, `bootstrap/evidence/gcc-4.0-native-cc1-build-frontier.json`, `src/bootstrap_parity.rs`, `openspec/specs/bootstrap/spec.md`.
- Verification: focused diagnostic eval/build, bootstrap parity tests/report, source-pin/blocker self-tests, OpenSpec validation, whitespace checks.
