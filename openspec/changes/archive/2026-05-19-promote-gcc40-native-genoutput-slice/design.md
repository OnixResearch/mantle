## Context

Commit `616ea830` promoted the bounded `genattrtab` native generator slice and archived `promote-gcc40-native-generator-frontier-slice`. The current `bootstrap/evidence/gcc-4.0-native-boundary.json` still marks `generator-empty-boundaries` with the derivation marker `Crunch GCC 4.0 empty-output source boundary: native genoutput promotion pending.`

This change intentionally stays narrow: it advances one selected generator member, records checked evidence, and preserves `gcc.4.0` as a blocking partial row.

## Approach

1. Confirm the current `genoutput` derivation marker and generated-output shape in `bootstrap/gcc-4.0.ncl`.
2. Define a bounded `genoutput` output contract with an explicit marker/symbol and digest or transcript evidence.
3. Update the GCC 4.0 native generator receipt/check path to accept the selected `genoutput` slice and reject stale or unsupported evidence.
4. Update the native frontier receipt to record `genoutput` as promoted and to name any remaining generator/native frontier blockers precisely.
5. Add positive and negative parity regressions, including an overclaim guard that keeps `gcc.4.0` `partial` and blocking.

## Non-Goals

- No claim of full native GCC 4.0 compiler correctness.
- No claim of full generator-family correctness.
- No live-bootstrap or Guix parity completion claim.
- No broad rewrite of the GCC 4.0 bridge.

## Risks

- Accidentally reclassifying `gcc.4.0` as complete would overclaim parity; tests must assert it remains partial/blocking.
- Reusing the `genattrtab` receipt shape without distinguishing `genoutput` could hide stale evidence; negative tests must cover selected-generator/schema/marker drift.
- OpenSpec archive may alter cumulative requirement text; inspect main spec diff before committing archive later.
