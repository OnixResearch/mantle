# record-gcc40-cparse-autohost-frontier-probe

## Why

The previous GCC 4.0 c-parse diagnostic receipt retired the copied `fd_bad` runtime branch as the recorded TinyCC/Mes frontier by proving a bounded instrumented compiler reaches fdopen/output-return markers. The remaining focused source-build attempt still fails compiling `c-parse.o`; the active diagnostic log already narrows part of that failure to `auto-host.h` macro interactions before `system.h`.

## What changes

- Record a v4 source-frontier receipt for the c-parse `auto-host.h` macro boundary.
- Require exact diagnostic markers for the autohost define window and the pass/fail probes around `NEED_64BIT_HOST_WIDE_INT`, `gid_t`, and `inline`.
- Preserve fail-closed validation and non-promotion semantics: this is diagnostic/source-frontier evidence only.

## Non-goals

- Do not promote `gcc.4.0` beyond `partial`.
- Do not claim native GCC 4.0 `cc1` or full source-build correctness.
- Do not remove or weaken prior v1-v3 source-frontier evidence contracts.
