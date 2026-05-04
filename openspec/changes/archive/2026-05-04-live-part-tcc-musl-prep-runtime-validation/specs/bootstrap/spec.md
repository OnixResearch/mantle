## ADDED Requirements

### Requirement: tcc musl prep runtime validation waits for prerequisite execution proof [r[bootstrap.part.tcc.musl.prep.runtime-validation]]
The system MUST keep the musl-prep TinyCC bridge runtime proof incomplete until prerequisite runtime blockers are resolved and the produced bridge/compiler carry-forward contract is smoke-tested.

#### Scenario: Bridge carry-forward artifacts are mandatory [r[bootstrap.part.tcc.musl.prep.runtime-validation.bridge-artifacts]]
- **GIVEN** `bootstrap/tcc-musl-prep.ncl` produces `bin/tcc`
- **WHEN** the Mes libc archive or Mes headers are missing
- **THEN** runtime validation remains incomplete

#### Scenario: Bridge smoke proves output [r[bootstrap.part.tcc.musl.prep.runtime-validation.smoke]]
- **GIVEN** prerequisite runtime blockers are resolved
- **WHEN** `bootstrap/tcc-musl-prep.ncl` builds and `tcc -v` succeeds
- **THEN** the evidence records output path, smoke results, fallback status, and leakage-scan results
