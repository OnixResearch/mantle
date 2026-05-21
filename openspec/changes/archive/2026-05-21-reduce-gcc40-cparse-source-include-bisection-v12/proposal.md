# Reduce GCC 4.0 c-parse source include bisection frontier v12

## Why

The v11 frontier proved that the focused `c-parse.o` make log's include-flood diagnostics are truncated and carry no actionable filename payload. The next useful step is a compact source-level include bisection that records which early `c-parse.c` include prefix still compiles under the TinyCC/Mes handoff and where the transition to failure begins, without expanding the active diagnostic back into the archived broad matrix.

## What

- Add bounded source-level include-prefix probes derived from `gcc/c-parse.c`'s ordered include list.
- Update the checked frontier receipt to schema `mantle-gcc40-native-cc1-source-frontier-reduction-v12` with the new include-prefix boundary.
- Keep `gcc.4.0` evidence-backed `partial` and preserve all non-claims about native compiler/source-build correctness.

## Impact

This is diagnostic/source-frontier reduction only. It does not promote GCC 4.0, native `cc1`, or full source-build correctness.
