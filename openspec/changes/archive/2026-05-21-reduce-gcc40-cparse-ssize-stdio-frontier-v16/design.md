## Context

v14 isolated the failure to `config.h + <stdio.h>`. v15 showed that undefining `ssize_t` alone makes that pair compile, while undefining `NEED_64BIT_HOST_WIDE_INT`, `gid_t`, `inline`, `rlim_t`, or `uid_t` alone still fails with the same truncated include-flood shape.

## Goals / Non-Goals

**Goals:**
- Keep the diagnostic addition compact.
- Distinguish `ssize_t` visibility before `<stdio.h>` from broader six-undef behavior.
- Preserve fail-closed parity evidence and partial status.

**Non-Goals:**
- Full native c-parse source-build correctness.
- Production build repair or promotion of `gcc.4.0` beyond `partial`.

## Decisions

### 1. Probe definition order, not a broad source rewrite

**Choice:** Add bounded probes that inspect/perturb only the generated `ssize_t` line around `<stdio.h>`.

**Rationale:** v15 already identified `ssize_t` as the narrowest passing single-undef. The smallest next proof is whether the generated definition poisons `<stdio.h>` before musl declares its own type.

**Alternative:** Attempt a direct `c-parse.o` repair. Rejected because the current evidence is still diagnostic/source-frontier and should move one seam at a time.

### 2. Keep parity partial and stale-evidence-gated

**Choice:** Require a new v16 schema and exact observed fragments while preserving v15 fragments.

**Rationale:** The evidence is useful only as a bounded frontier; missing or stale markers must not silently promote GCC 4.0.

## Risks / Trade-offs

- **Diagnostic script growth:** Keep probes compact and run `mantle eval` before any long build.
- **Archive spec drift:** After archive, inspect `openspec/specs/bootstrap/spec.md` and restore cumulative GCC ladder text if OpenSpec collapses it.
