## ADDED Requirements

### Requirement: First GNU make pass is amd64 executable [r[bootstrap.part.make.3.82.amd64.execution]]
Crunch MUST build `bootstrap/make-tcc.ncl` into a `make 3.82` output that executes basic Makefiles on amd64.

The output MUST include `bin/make`, report `GNU Make 3.82`, execute a simple Makefile target successfully, and reject a missing target with a controlled nonzero exit rather than a signal or segmentation fault. Source-level repair completion evidence MUST include a source-pin audit transcript and either direct successful runtime transcripts or an active runtime-validation successor that records the long-build, smoke, and host-leakage proof still required before downstream completion claims.

#### Scenario: Simple Makefile runs [r[bootstrap.part.make.3.82.amd64.execution.scenario.simple-makefile]]

- GIVEN `bootstrap/make-tcc.ncl` has been built with the documented bootstrap environment
- WHEN the produced `bin/make -f Makefile` runs a Makefile whose `all` target echoes `make-smoke-ok`
- THEN stdout contains `make-smoke-ok`
- AND the process exits successfully

#### Scenario: Missing target fails cleanly [r[bootstrap.part.make.3.82.amd64.execution.scenario.missing-target]]

- GIVEN the same produced `bin/make`
- WHEN it is asked to build a missing target
- THEN it exits nonzero
- AND the exit status is not a signal-derived segmentation fault

#### Scenario: Evidence is complete [r[bootstrap.part.make.3.82.amd64.execution.scenario.evidence-complete]]

- GIVEN the repair is marked complete
- WHEN reviewers inspect this change
- THEN they can find the source-pin audit transcript
- AND they can find either successful build, smoke, and host-leakage transcripts
- OR they can find an active runtime-validation successor that owns those remaining proof transcripts
