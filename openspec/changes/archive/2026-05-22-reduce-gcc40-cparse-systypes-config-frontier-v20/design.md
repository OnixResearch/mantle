## Context

v18 established that `config.h + <stdio.h>` passes when `<sys/types.h>` appears before generated `config.h`. v19 tested an append-only generated-config suppression (`#undef ssize_t`) against the real `c-parse.o` make target and found the same rc=2/two-line include-flood frontier.

## Goals / Non-Goals

**Goals:**
- Add a single bounded real `make -C gcc c-parse.o` probe with generated `config.h` adjusted by preincluding `<sys/types.h>` before the existing generated content.
- Preserve the original baseline make frontier and non-promotion semantics.

**Non-Goals:**
- Do not patch production `bootstrap/gcc-4.0.ncl` in this slice.
- Do not claim complete GCC 4.0 native `cc1` correctness.

## Decisions

### 1. Preinclude-only generated config adjustment

**Choice:** Copy generated `gcc/config.h`, rewrite it to prepend `#include <sys/types.h>` followed by the original content, run the same focused make target, capture rc/log summary, then restore the original config before the baseline make probe.

**Rationale:** This directly tests the v18 order insight in the real make path while staying compact enough for the diagnostic derivation's argument-size constraint.

**Alternative:** Patch the GCC source include order or production derivation. Rejected for this slice because the immediate need is to prove whether the minimal generated-config preinclude advances the real frontier before touching production bootstrap code.

## Risks / Trade-offs

- The preinclude probe may expose a later frontier or still fail at the same shape; both outcomes are useful if recorded precisely and kept non-promoting.
- Diagnostic script size is near argv/env limits, so preserve archived prior probes as receipt text and keep active instrumentation compact.
