## Why

The v17 GCC 4.0 native `cc1` c-parse frontier showed that both pre-`<stdio.h>` macro and typedef definitions of `ssize_t` reproduce the two-line truncated include-flood failure, while a post-`<stdio.h>` macro definition succeeds. The next smallest seam is whether making the host `ssize_t` typedef visible before GCC's generated `config.h` changes the outcome.

## What Changes

- Add a compact v18 source-frontier probe that includes `<sys/types.h>` before `config.h`, then includes `<stdio.h>`.
- Update the GCC 4.0 native `cc1` source-frontier receipt, parity validation, and spec scenarios with the observed v18 result.
- Preserve `gcc.4.0` as `partial`; this is diagnostic frontier evidence, not native compiler correctness.

## Impact

- Files: `bootstrap/diag-gcc40-c-parse-boundary.ncl`, `bootstrap/evidence/gcc-4.0-native-cc1-build-frontier.json`, `src/bootstrap_parity.rs`, `openspec/specs/bootstrap/spec.md`.
- Testing: focused parity tests, CLI parity report, source-pin/blocker checks, `openspec validate --all --strict`, and `git diff --check`.
