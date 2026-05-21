# Design

## Scope

The slice is intentionally narrow: `genrecog` must produce a deterministic bounded recognition source fragment with a named marker/symbol and BLAKE3 digest/transcript evidence. Existing `genattrtab`, `genoutput`, and `genemit` evidence remains regression coverage.

## Non-goals

- No full native generator correctness proof.
- No full native GCC 4.0 compiler/source-build proof.
- No change to `gcc.4.0` partial status.
- No further c-parse diagnostic/log micro-instrumentation.

## Validation

Parity validation should require:

- schema bump from native-generator v3 to v4;
- `selected_generators` includes `genrecog` in addition to existing generator slices;
- a `bounded_outputs.genrecog` object with source-bound derivation marker, bounded contract, output fragment, and recomputed BLAKE3 transcript/output digests;
- forbidden stale empty-genrecog boundary markers are absent;
- parity report still leaves `gcc.4.0` partial and blocks full correctness claims.
