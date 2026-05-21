# Clarify GCC 4.0 c-parse include-flood truncation frontier v11

## Why

The current v10 frontier proves the focused native `c-parse.o` make attempt reaches a repeated include diagnostic flood, but the captured log still looks actionable even though the payload is truncated to repeated `In file included from` text. Future drains need durable evidence that log scraping alone cannot identify a source filename and that the next useful work is source-level include bisection or a real source-build fix.

## What

- Record bounded truncation/payload markers for the two include-flood log lines.
- Update the checked frontier receipt to schema `mantle-gcc40-native-cc1-source-frontier-reduction-v11`.
- Keep `gcc.4.0` evidence-backed `partial` and preserve all non-claims about native compiler/source-build correctness.

## Impact

This is diagnostic/source-frontier clarification only. It does not promote GCC 4.0, native `cc1`, or full source-build correctness.
