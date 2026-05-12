## Context

The parity report already rejects standalone placeholder markers, but `gcc.4.0` intentionally carries many bridge/generator markers from the pass1 graph-completion milestone. The next useful seam is to make those markers explicit and fail closed on drift.

## Decisions

### Checked inventory receipt

The receipt records schema, derivation, status, and the exact set of standalone marker occurrences by line and marker string. The validator recomputes markers from `bootstrap/gcc-4.0.ncl` and requires exact equality.

### Partial, not complete

A matching receipt downgrades `gcc.4.0` from unclassified `placeholder` to evidence-backed `partial`. It remains a blocker because the receipt is inventory-only and does not prove native `cc1`, `xgcc`, `cpp`, or full `libgcc` correctness.

## Risks / Trade-offs

- Line-number inventory is intentionally strict; harmless refactors that move markers must update the receipt. This is acceptable because the receipt is a drift detector.
- This change does not remove any GCC bridge behavior; it only makes the current debt explicit.
