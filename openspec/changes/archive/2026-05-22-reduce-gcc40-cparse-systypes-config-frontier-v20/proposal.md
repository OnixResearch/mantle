## Why

v18 showed that including `<sys/types.h>` before generated `config.h` makes the compact `config.h + <stdio.h>` probe pass. v19 showed that merely appending `#undef ssize_t` to generated `gcc/config.h` does not advance the real `make -C gcc c-parse.o` frontier. The next smallest production-shaped source adjustment is to make generated `gcc/config.h` preinclude `<sys/types.h>` before its existing content and run the real make target.

## What Changes

- Add one compact diagnostic make probe that prepends `#include <sys/types.h>` to generated `gcc/config.h` before rebuilding `c-parse.o`, then restores the original config for the unchanged baseline frontier.
- Record the new sys/types-preinclude real make frontier in the source-frontier receipt and fail-closed parity checks.
- Keep `gcc.4.0` partial; this is still diagnostic source-frontier evidence, not native GCC correctness.

## Impact

- Files: `bootstrap/diag-gcc40-c-parse-boundary.ncl`, `bootstrap/evidence/gcc-4.0-native-cc1-build-frontier.json`, `src/bootstrap_parity.rs`, `openspec/specs/bootstrap/spec.md`.
- Verification: focused diagnostic eval/build, bootstrap parity tests/report, source-pin/blocker self-tests, OpenSpec validation, whitespace checks.
