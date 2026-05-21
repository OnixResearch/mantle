## Context

The active GCC 4.0 native source frontier is the `config.h` + `<stdio.h>` interaction in the TinyCC/Mes c-parse diagnostic path. v15 isolated `ssize_t`; v16 showed that a pre-stdio `#define ssize_t int` reproduces the include-flood failure while defining it after `<stdio.h>` succeeds.

## Goals / Non-Goals

**Goals:**
- Record whether the remaining seam is specifically a pre-stdio macro collision or any pre-stdio `ssize_t` definition.
- Keep probes compact enough to stay below the diagnostic derivation argument-size frontier.
- Keep parity fail-closed on stale or overclaiming evidence.

**Non-Goals:**
- No full GCC 4.0 native `cc1` source-build correctness claim.
- No broad rewrite of GCC config generation or installed `cc1` semantics.

## Decisions

### 1. Add only bounded source-level probes

**Choice:** Add a v17 probe family adjacent to the v16 `ssize_t` order probes.
**Rationale:** This is the smallest increment that distinguishes macro substitution from typedef/order behavior without expanding the real source-build attempt.
**Alternative:** Patch the production config generation immediately. Rejected until the macro-vs-typedef behavior is recorded as fail-closed evidence.

### 2. Preserve partial status

**Choice:** Update evidence schema and parity requirements but keep `gcc.4.0` partial.
**Rationale:** These probes only explain a diagnostic frontier; they do not prove the native compiler/generator pipeline.

## Risks / Trade-offs

- **Script-size pressure** → Keep markers terse and reuse the existing probe helper.
- **OpenSpec archive drift** → After archive, restore the base spec if the long GCC ladder requirement collapses, then reapply only the v17 scenario block.
