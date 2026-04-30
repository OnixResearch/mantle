## ADDED Requirements

### Requirement: Make 3.82 amd64 runtime validation completes [r[bootstrap.part.make.3.82.amd64.runtime-validation]]
Crunch MUST preserve runtime proof for the repaired `bootstrap/make-tcc.ncl` output before the first GNU make amd64 repair is treated as complete.

The proof MUST include a long-budget build transcript, the produced output path or concrete failure diagnostics, version smoke evidence for `GNU Make 3.82`, a simple Makefile positive smoke, a missing-target negative smoke that fails without a signal/segfault, and a host-leakage scan over the derivation, transcript, and output.

#### Scenario: Long build produces make output [r[bootstrap.part.make.3.82.amd64.runtime-validation.scenario.build-output]]
- GIVEN the parent repair source state
- WHEN `crunch build bootstrap/make-tcc.ncl` runs with the documented bootstrap environment and a long validation budget
- THEN the transcript is preserved
- AND either a make output path is recorded or concrete failure diagnostics are recorded

#### Scenario: Make runtime smokes are complete [r[bootstrap.part.make.3.82.amd64.runtime-validation.scenario.smoke]]
- GIVEN a produced make output path
- WHEN the successor runs version, positive Makefile, and missing-target negative checks
- THEN version output contains `GNU Make 3.82`
- AND the positive Makefile prints `make-smoke-ok`
- AND the missing target exits nonzero without a signal-derived segmentation fault

#### Scenario: Host leakage is scanned [r[bootstrap.part.make.3.82.amd64.runtime-validation.scenario.host-leakage]]
- GIVEN the build transcript and output metadata
- WHEN the host-leakage scan runs
- THEN undeclared host tools, host paths, and environment-derived inputs are either absent or recorded as concrete failures
