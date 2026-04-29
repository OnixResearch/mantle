## ADDED Requirements

### Requirement: Tinycc Mes self-compile blocker is resolved
Crunch MUST resolve the `BufferedFile`/first-self-compile blocker before `live-part-tinycc-0-9-26` can claim successful build evidence.
ID: bootstrap.part.tinycc.0.9.26.selfcompile

#### Scenario: Full tinycc output contract is required evidence

- GIVEN `bootstrap/tinycc-mes.ncl` builds `tcc-mes`
- WHEN the blocker is marked resolved
- THEN the evidence MUST show `tcc-mes` compiles at least `tcc-boot0` without segfaulting
- AND `crunch build bootstrap/tinycc-mes.ncl` MUST finish successfully
- AND the produced output MUST include executable `bin/tcc` and `bin/tcc-0.9.26`
- AND the produced compiler MUST compile a trivial C program
- AND `tcc-mes -version` alone MUST NOT be accepted as the smoke boundary
