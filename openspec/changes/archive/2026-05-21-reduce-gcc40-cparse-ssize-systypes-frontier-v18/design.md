## Context

v15 narrowed the `config.h + <stdio.h>` failure to the generated `ssize_t` definition. v16 showed pre-stdio definition order reproduces the failure, while post-stdio redefinition succeeds. v17 showed the failure is not macro-substitution-only because a pre-stdio typedef also fails.

## Goals / Non-Goals

Goals:
- Add one compact diagnostic probe for host typedef visibility before `config.h`.
- Record the observed result fail-closed in the source-frontier receipt and parity tests.

Non-goals:
- Do not claim full GCC 4.0 native `cc1` correctness.
- Do not add a broad include-bisection matrix or increase script size materially.

## Decisions

### Host typedef preinclude probe

Choice: create a temporary config header that includes `<sys/types.h>` before `config.h`, then reuse the existing `run_cparse_stdio_probe` helper so the probe remains compact.

Rationale: this directly tests whether making the host `ssize_t` typedef visible before GCC's generated `config.h` changes the failure shape, without duplicating compile/error-summary plumbing.

Alternative rejected: add a new direct compile helper or broad header-order matrix. That risks the known bwrap argv/env size limit around the diagnostic derivation.

## Risks / Trade-offs

- The diagnostic script is close to the argument-size limit; keep v18 to one additional probe and prune chatter if needed.
- The result is diagnostic-only and must keep `gcc.4.0` partial unless a later slice proves native source-build correctness.
