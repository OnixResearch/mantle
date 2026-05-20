# record-gcc40-cparse-generated-header-frontier-probe

## Why

The v4 GCC 4.0 c-parse source-frontier receipt narrowed the focused diagnostic to the `auto-host.h` macro window. A fresh bounded diagnostic run shows that applying the six known `auto-host.h` undefines advances through `system.h`, `coretypes.h`, `tm.h`, and several generated-header include forms, with the next stable matrix around `insn-modes.h`, `machmode.h`, and early `tree.h` prefix probes.

## What changes

- Record a v5 source-frontier receipt for the c-parse generated-header boundary after the v4 autohost macro seam.
- Require exact diagnostic markers for the six-undef autohost/full-config probes and representative `insn-modes.h`, `machmode.h`, and `tree.h` prefix outcomes.
- Preserve fail-closed validation and non-promotion semantics: this is diagnostic/source-frontier evidence only.

## Non-goals

- Do not promote `gcc.4.0` beyond `partial`.
- Do not claim native GCC 4.0 `cc1` or full source-build correctness.
- Do not remove or weaken prior v1-v4 source-frontier evidence contracts.
