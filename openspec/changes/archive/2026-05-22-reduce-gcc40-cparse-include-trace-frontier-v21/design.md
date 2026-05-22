## Context

The active frontier is the GCC 4.0 native `c-parse.o` compile under the TinyCC/Mes handoff. v19 and v20 tried narrow generated-config order changes and both preserved the same rc=2/two-line truncated include-flood failure. v11 already established the make-log flood lines themselves carry no filename payload.

## Goals

- Capture bounded include-trace payload from the same compiler command without relying on the truncated fatal diagnostic text.
- Keep the diagnostic derivation compact enough for the bwrap argument-size limit.
- Keep all claims frontier-only and fail-closed through parity validation.

## Non-Goals

- Fixing the GCC source-build failure in this slice.
- Replacing the baseline real make probe.
- Claiming native GCC 4.0 `cc1` correctness.

## Decisions

### Preprocessor line-marker trace

Run `gcc40-cc -E` against `gcc/c-parse.c` with the same GCC make compile include/define surface, redirect stdout/stderr to bounded temp logs, and summarize only compact deterministic facts into the receipt:

- return code
- number of `# <line> "<file>"` line markers emitted
- the first and last observed line-marker file payloads
- whether a concrete filename payload was recovered

This recovers include payload from TCC's preprocessor stream rather than the recursively duplicated fatal diagnostic prefix.

### Preserve make frontier

Keep the existing focused `make -C gcc c-parse.o` baseline after the trace probe and continue requiring the unchanged rc=2/include-flood evidence. The trace is diagnostic context, not a success condition for GCC.

## Risks

- TCC may emit no line markers before failing. The receipt must record that outcome explicitly and still preserve the make-frontier failure.
- Adding instrumentation can push the derivation toward the argument-size limit. Keep the probe small and summarize with `awk`/`grep` rather than dumping logs.
