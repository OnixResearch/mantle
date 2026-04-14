## ADDED Requirements

### Requirement: Determinism regression coverage

The repo MUST provide automated regression coverage that reruns representative
builds while perturbing ambient host state and checks that crunch either
produces identical results or fails for an explicit strict-mode blocker.

At minimum the regression matrix MUST vary `HOME`, `PATH`, `USER`, `TZ`,
`LANG`, `TMPDIR`, current working directory, and umask.

#### Scenario: Ambient host changes do not perturb output identity

- GIVEN a representative derivation selected for the determinism harness
- WHEN the harness reruns it under multiple ambient host-state combinations
- THEN the successful runs produce the same output digest
- AND the emitted hermeticity audit facts stay identical across those runs

#### Scenario: Strict-mode blocker remains stable under ambient-state variation

- GIVEN a representative strict-mode build that is expected to fail for a known blocker
- WHEN the harness reruns it under multiple ambient host-state combinations
- THEN the same blocker class is reported each time
- AND the result does not silently degrade into a different weaker execution path
