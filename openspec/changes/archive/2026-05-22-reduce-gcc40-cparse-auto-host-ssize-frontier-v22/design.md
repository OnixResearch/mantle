## Context

The active GCC 4.0 `c-parse.o` source frontier is diagnostic-only. v21 recovered the concrete collision path but left the real make target at rc=2 with two truncated include-flood lines. The next smallest test is not another `config.h` ordering change; it is a direct generated `auto-host.h` seam probe.

## Goals

- Test the narrowest generated-header patch suggested by v21: remove/comment the generated `#define ssize_t ...` line in `gcc/auto-host.h` before the real `c-parse.o` make target.
- Preserve the baseline make markers and record whether the include-flood disappears or the frontier advances.
- Keep receipts non-promoting and fail-closed.

## Non-Goals

- Permanent production fix to GCC 4.0 configuration.
- Native compiler correctness or full source-build promotion.
- Large new diagnostic matrices.

## Decisions

### 1. Patch `auto-host.h`, not `config.h`

**Choice:** Copy `gcc/auto-host.h`, replace the `#define ssize_t` line with a commented diagnostic marker, run `make -C gcc c-parse.o`, then restore the file.

**Rationale:** v19 appended `#undef ssize_t` to `config.h` and v20 prepended `<sys/types.h>` to `config.h`; neither advanced the real make target. v21 shows the concrete payload originates in `auto-host.h`, so direct seam mutation is the smallest new hypothesis.

### 2. Record outcome, not success assumptions

**Choice:** The probe captures rc, compile-command presence, include-flood line count, and whether `c-parse.o` exists.

**Rationale:** If the probe advances, the next frontier may be a different compile error rather than a successful object build. If it does not advance, the receipt should prevent repeating this seam.

## Risks

- The generated header line may have a different spelling. Mitigation: record whether replacement occurred and fail closed in parity markers.
- A successful `c-parse.o` could expose later failures; the v22 receipt must describe the observed frontier exactly.