## Context

v15 showed that undefining only `ssize_t` after generated `config.h` is enough for compact `config.h + <stdio.h>` probes to pass. v18 showed that preincluding `<sys/types.h>` before `config.h` also passes. The real make frontier still fails at `c-parse.o` with the truncated include-flood shape.

## Goals / Non-Goals

**Goals:**
- Add a single bounded real `make -C gcc c-parse.o` probe with generated `config.h` adjusted by appending `#undef ssize_t`.
- Preserve the original baseline make frontier and non-promotion semantics.

**Non-Goals:**
- Do not patch production `bootstrap/gcc-4.0.ncl` in this slice.
- Do not claim complete GCC 4.0 native `cc1` correctness.

## Decisions

### 1. Append-only config adjustment

**Choice:** Copy generated `gcc/config.h`, append `#undef ssize_t`, run the same focused make target, capture rc/log summary, then restore the original config before the baseline make probe.

**Rationale:** This mirrors the v15 successful single-undef order while staying compact and avoiding broader configure/header rewrites.

**Alternative:** Replace `config.h` with a `<sys/types.h>` preinclude wrapper. Rejected for this slice because append-only `#undef ssize_t` is the smaller source-fix candidate and directly tests generated-config suppression.

## Risks / Trade-offs

- The adjusted make probe may expose a later frontier or still fail at the same shape; both are useful if recorded precisely and kept non-promoting.
- Diagnostic script size is near argv/env limits, so keep the probe compact and reuse existing log-summary patterns.
