# Tasks

## Evaluation and conversion rails

- [ ] [serial] Parameterize `tests/examples_eval.rs` or a new inventory test over the catalog so every eligible Nickel example is evaluated and converted according to its declared support tier. r[examples.validation_matrix]
- [ ] [serial] Add negative evaluation tests for malformed Nickel, missing generated seed material, bad project selectors, and intentionally failing diagnostic examples. r[examples.validation_matrix] r[examples.output_execution]

## Build and output rails

- [ ] [serial] Expand `tests/examples_build.rs` with fast/offline smoke builds that use temp store/state roots and explicit Linux/bwrap capability skips. r[examples.validation_matrix]
- [ ] [serial] Add output execution or inspection assertions for runnable examples, multi-output layouts, and project check outputs. r[examples.output_execution]
- [ ] [serial] Keep heavyweight real-crate/bootstrap examples behind ignored tests or explicit scripts with documented commands and expected evidence locations. r[examples.validation_matrix]

## Verification

- [ ] [serial] Run focused examples eval tests and record output. r[examples.validation_matrix]
- [ ] [serial] Run focused examples build/output tests on a capable host or record explicit capability blockers. r[examples.validation_matrix] r[examples.output_execution]
- [ ] [serial] Run `cairn validate --root .` and the tasks gate, then archive only after completed tasks cite durable evidence. r[examples.validation_matrix] r[examples.output_execution]
