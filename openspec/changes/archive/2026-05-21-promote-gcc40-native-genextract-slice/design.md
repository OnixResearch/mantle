# Design

## Scope

Promote only `genextract` from an empty boundary marker to a bounded output-slice marker. Do not broaden into real RTL extraction semantics, native source-build success, or GCC 4.0 parity completion.

## Evidence Model

The derivation emits a deterministic fragment with:

- `GCC40_GENEXTRACT_BOUNDED`
- `gcc40_genextract_bounded_output_slice`
- explicit non-claim wording that full generator correctness remains pending

`bootstrap/evidence/gcc-4.0-native-generator-slice.json` becomes schema `mantle-gcc40-native-generator-slice-v5`, adds `genextract` to `selected_generators`, stores transcript/output BLAKE3 digests, and forbids the stale empty-extraction boundary markers.

## Validation

Parity validation must require the new selected generator and output markers, reject stale schema/selected-generator/digest/contract drift, and keep `gcc.4.0` partial. The native boundary receipt may list the bounded genextract slice but must retain `generator-bounded-outputs` as a frontier blocker.
