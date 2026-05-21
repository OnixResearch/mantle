# Record GCC 4.0 c-parse compact make frontier probe

## Summary

Compact the diagnostic-only GCC 4.0 c-parse source-frontier derivation so it stays below the host builder argument-size limit while preserving the current focused `c-parse.o` make-error boundary as checked frontier evidence.

## Motivation

The v7 receipt records the real focused `c-parse.o` make-error boundary, but the diagnostic derivation still carries a large archived generated-header/autohost probe matrix. Recent diagnostic build logs can fail before the frontier with `Argument list too long (os error 7)`, which reduces the usefulness of the active diagnostic probe. The completed v5/v6 sweep details already live in checked evidence; the active derivation should keep only compact source-resident markers needed to reproduce the current make-error frontier.

## Scope

- Replace the active archived broad generated-header/autohost diagnostic matrix with compact documented marker lines and the focused make-error capture.
- Advance the source-frontier receipt schema to v8 and require the compact diagnostic marker contract.
- Preserve v5/v6/v7 observed frontier facts as evidence text, not as required source-resident diagnostic probe calls.
- Keep `gcc.4.0` evidence-backed `partial` and explicitly non-promoted.

## Non-goals

- Proving native GCC 4.0 compiler, generator, or source-build correctness.
- Promoting `gcc.4.0` to complete.
- Reintroducing the archived broad c-parse bisection matrix.
