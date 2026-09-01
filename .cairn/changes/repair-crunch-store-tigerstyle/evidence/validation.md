# Validation evidence

## Baseline

- Repository Tiger derivation: status 1, with 139 location-backed findings in
  the first failing `crunch-store` target surface.
- Focused package command: status 1, with 183 findings across 13 files.
- Pre-change tests: 357 unit tests and 2 integration tests pass.

The focused package result is the stronger implementation baseline. Completion
requires zero findings there and in the repository derivation.

## Review checkpoint

- **Question:** Can strict conformance be restored without changing store
  meaning or weakening the gate?
- **Inspected evidence:** both Tiger baselines, source ownership boundaries,
  accepted store lifecycle requirements, and 359 passing pre-change tests.
- **Decision:** use local invariant repairs first and extract helpers only where
  function length or interface shape requires it. Reject allowances and scope
  reductions.
- **Owner:** `repair-crunch-store-tigerstyle`.
- **Next action:** repair the 13 source files in bounded families and rerun the
  focused command after each family.
