# Promote GCC 4.0 native genextract slice

## Why

GCC 4.0 still carries bounded native generator frontier debt. `genattrtab`, `genoutput`, `genemit`, and `genrecog` now have checked bounded output slices, but `genextract` is still represented by an older empty-extraction boundary. Replacing that with a deterministic bounded generated extraction output gives better evidence without claiming full generator correctness.

## What Changes

- Replace the old empty `genextract` source-boundary output with a checked bounded extraction output marker in `bootstrap/gcc-4.0.ncl`.
- Extend the native-generator receipt to schema v5 with `genextract` output/transcript/digest evidence.
- Extend parity validation, fixtures, and negative drift checks so stale `genextract` empty-boundary evidence fails closed.
- Keep `gcc.4.0` evidence-backed `partial`; do not claim native/full generator or compiler correctness.

## Impact

This is a narrow bootstrap evidence slice. It improves the generator frontier map while preserving all existing blockers and non-claim semantics.
