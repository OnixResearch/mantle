## ADDED Requirements

### Requirement: make 3.82 runtime validation waits for amd64 repair [r[bootstrap.part.make.3.82.runtime-validation]]
The system MUST keep make 3.82 runtime proof incomplete until the amd64 varargs repair is complete and `bootstrap/make-tcc.ncl` executes a simple Makefile successfully.

#### Scenario: Version output alone is insufficient [r[bootstrap.part.make.3.82.runtime-validation.version-insufficient]]
- **GIVEN** the produced make binary reports GNU Make 3.82 with `--version`
- **WHEN** simple Makefile execution still segfaults or fails
- **THEN** runtime validation remains incomplete

#### Scenario: Runtime proof closes after repair [r[bootstrap.part.make.3.82.runtime-validation.after-repair]]
- **GIVEN** `repair-make-tcc-amd64-varargs` is complete
- **WHEN** `bootstrap/make-tcc.ncl` builds and runs a simple Makefile successfully
- **THEN** the evidence records output path, smoke results, fallback status, and leakage-scan results
