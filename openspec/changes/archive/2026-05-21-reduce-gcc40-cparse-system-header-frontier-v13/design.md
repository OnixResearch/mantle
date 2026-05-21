## Context

Source-frontier v12 proved the first failing `gcc/c-parse.c` include prefix is `config.h` + `system.h`. GCC 4.0 `system.h` directly includes standard headers plus GCC-local `hwint.h`, `filenames.h`, and `libiberty.h`. The active diagnostic must stay compact because prior broad c-parse matrices hit Linux argument-size limits.

## Goals / Non-Goals

**Goals:**
- Record ordered direct `system.h` includes in the diagnostic and receipt.
- Probe a small set of direct include prefixes to identify whether the failure is immediate at the earliest `system.h` include or later.
- Preserve v12 source-prefix and v11 make-log truncation evidence.
- Add fail-closed validation for stale or missing v13 evidence.

**Non-Goals:**
- Do not fix GCC 4.0 native `cc1` source build in this slice.
- Do not claim native compiler, full generator, or full source-build correctness.
- Do not reintroduce broad archived v5-v7 matrices or expand make-log scraping.

## Decisions

### 1. Direct include-prefix probes only

**Choice:** Generate probes that include normalized `config.h`, then direct headers extracted from `gcc/system.h` in source order.
**Rationale:** v12 already isolated the outer include seam to `system.h`; direct nested prefixes are the smallest next inspectable boundary.
**Alternative:** Preprocess full `system.h` or scrape the existing make log further. Rejected because v11 showed the log is truncated and broad preprocessing risks noisy/non-compact evidence.

### 2. Fail-closed receipt schema bump

**Choice:** Bump the source-frontier reduction schema to v13 and require explicit nested-prefix fragments.
**Rationale:** Prevents stale v12 evidence from passing after the diagnostic changes.

## Risks / Trade-offs

**Argument-size growth** → Keep probes few and marker text compact.
**False promotion signal** → Keep status `frontier-only`, preserve parity `partial`, and require non-claim wording.
