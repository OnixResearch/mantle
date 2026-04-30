## ADDED Requirements

### Requirement: First GNU make pass is amd64 executable
Crunch MUST build `bootstrap/make-tcc.ncl` into a `make 3.82` output that executes basic Makefiles on amd64.
ID: bootstrap.part.make.3.82.amd64.execution

The output MUST include `bin/make`, report `GNU Make 3.82`, execute a simple Makefile target successfully, and reject a missing target with a controlled nonzero exit rather than a signal or segmentation fault. Completion evidence MUST include a source-pin audit transcript, successful `crunch build bootstrap/make-tcc.ncl` transcript, version smoke transcript, simple Makefile positive transcript, missing-target negative transcript, and host-leakage scan transcript.

#### Scenario: Simple Makefile runs

- GIVEN `bootstrap/make-tcc.ncl` has been built with the documented bootstrap environment
- WHEN the produced `bin/make -f Makefile` runs a Makefile whose `all` target echoes `make-smoke-ok`
- THEN stdout contains `make-smoke-ok`
- AND the process exits successfully

#### Scenario: Missing target fails cleanly

- GIVEN the same produced `bin/make`
- WHEN it is asked to build a missing target
- THEN it exits nonzero
- AND the exit status is not a signal-derived segmentation fault

#### Scenario: Evidence is complete

- GIVEN the repair is marked complete
- WHEN reviewers inspect this change
- THEN they can find the source-pin audit transcript
- AND they can find the successful build transcript
- AND they can find positive and negative smoke transcripts
- AND they can find the host-leakage scan transcript
