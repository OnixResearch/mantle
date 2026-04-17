## ADDED Requirements

### Requirement: Benchmark docs match the checked-in workflow

The repo MUST document the checked-in benchmark workflow in README-linked docs.

That documentation MUST name the smoke benchmark entry point, the full suite
entry point, and the baseline-vs-fresh comparison entry point.

That documentation MUST describe sparse phase metrics as omission, not as
implicit zeroes.

#### Scenario: Reader finds the three benchmark entry points

- GIVEN a contributor wants to run the checked-in benchmark workflow
- WHEN they follow the README benchmark link into the focused benchmark doc
- THEN they can find the smoke command, the full-suite command, and the
  comparison command
- AND those commands match the checked-in benchmark tooling

#### Scenario: Sparse metrics stay explicit in docs

- GIVEN a contributor reads the benchmark comparison guidance
- WHEN they learn how missing phase metrics are reported
- THEN the doc says missing metrics are explicit omissions
- AND it does not imply that absent metrics should be read as zero