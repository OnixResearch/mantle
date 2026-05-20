# Proposal: record GCC 4.0 c-parse make-error frontier v7

## Why

The v6 source-frontier receipt narrowed the diagnostic boundary to the real `c-parse.o` make target after generated-header sweep probes. The remaining build failure currently exits through `make` with truncated/repeated include diagnostics, which is less inspectable than the earlier named probe matrix.

## What changes

- Wrap the diagnostic derivation's focused `c-parse.o` make attempt in a bounded capture that records the make rc and a deterministic stderr/stdout head/tail marker before returning the same failing status.
- Update the source-frontier receipt to v7 with exact diagnostic markers for the captured make-error boundary.
- Update parity validation/spec tests to accept only the v7 evidence and keep stale v6-only evidence fail-closed.

## Non-goals

- Do not promote `gcc.4.0`.
- Do not claim native GCC 4.0 c-parse/cc1 source-build correctness.
- Do not broaden the diagnostic matrix or retry full bootstrap.
