# Record GCC 4.0 c-parse make-log summary frontier probe

## Why

The v8 compact diagnostic preserves the focused `c-parse.o` make-error boundary, but the bounded tail still hides useful, deterministic log shape details behind repeated include diagnostics. Capturing compact log-summary markers gives future source-frontier work stable evidence for where the real make attempt starts and how large the failure log is, without reintroducing the broad generated-header probe matrix or claiming native GCC correctness.

## What Changes

- Add source-resident diagnostic markers for the focused `c-parse.o` make log compile-command presence, line count, and byte count.
- Advance the nested source-frontier receipt to v9 and require the new markers fail-closed.
- Preserve the existing make-error boundary and keep `gcc.4.0` `partial`.
