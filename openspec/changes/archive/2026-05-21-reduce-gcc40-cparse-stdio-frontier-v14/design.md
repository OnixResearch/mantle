## Context

v13 showed the first nested `system.h` failing boundary is adding `<stdio.h>` after `config.h`, `<stdarg.h>`, and `<stddef.h>`. The next slice should avoid another broad matrix and keep diagnostic size small.

## Goals

- Identify a narrower `stdio.h`-related source frontier using compact diagnostic probes.
- Preserve v11-v13 evidence in the receipt and parity validation.
- Keep the result diagnostic/source-frontier only.

## Non-Goals

- Fix GCC 4.0 native `c-parse.o` compilation.
- Claim native GCC 4.0 compiler/source-build correctness.
- Re-run unbounded make-log scraping or add large generated header matrices.

## Decisions

### 1. Probe stdio as a direct include/macro seam

**Choice:** Add small diagnostic probes around `config.h` + `stdarg.h`/`stddef.h` + `stdio.h`, including bounded variants that help distinguish whether stdio alone, prerequisite headers, or the combined prefix triggers the flood.

**Rationale:** v13 already identified `stdio.h` as the first new direct include. A focused v14 seam creates better evidence without growing the diagnostic derivation into another archived matrix.

### 2. Keep validation fail-closed

**Choice:** Bump the receipt schema to v14 and require observed v14 fragments in `src/bootstrap_parity.rs`.

**Rationale:** The parity report must not silently accept stale v13 evidence or overclaim `gcc.4.0` completeness.

## Risks / Trade-offs

- The probe may still only localize the seam rather than produce a fix. Mitigation: phrase receipt/spec as source-frontier evidence only.
- Diagnostic script size may grow. Mitigation: add only compact probes and check eval script size before builds.
