# Design: GCC 4.0 native cc1 pointer-deref slice

## Scope

The slice is intentionally narrow: a single C input with `int value`, `int *slot = &value`, a dereference store `*slot = x + 4`, dereference load `return *slot + value`, and the existing GCC-shaped `cc1` invocation.

## Evidence model

The existing `bootstrap/evidence/gcc-4.0-native-cc1-arithmetic.json` remains the cumulative semantic receipt. It moves from v6 to v7, makes `pointer-deref-v7` the selected slice, and records the prior struct-field v6 as a regression.

Validation remains fail-closed:

- exact receipt schema and selected slice
- exact bounded source fragments
- expected no-TinyCC-delegation derivation marker
- expected object marker and BLAKE3 transcript/output digests
- preservation of all prior semantic slice regressions

## Claim boundary

The parity row wording may mention pointer-deref alongside prior bounded slices, but the row must remain partial. The receipt and tests must continue to state that native compiler correctness is not proven.
