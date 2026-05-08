## ADDED Requirements

### Requirement: gcc-4.0 runtime validation completes after binutils-tcc [r[bootstrap.gcc40.runtime-validation]]
The system MUST validate gcc-4.0.4 only after the binutils-tcc runtime evidence is available or explicitly blocked.

#### Scenario: Build transcript records transition evidence [r[bootstrap.gcc40.runtime-validation.transcript]]
- **GIVEN** the binutils-tcc output is available
- **WHEN** `bootstrap/gcc-4.0.ncl` is built
- **THEN** the transcript records command, provider, exit status, output path or failure class, fallback status, and placeholder rejection

#### Scenario: Compiler smoke tests prove output [r[bootstrap.gcc40.runtime-validation.smoke]]
- **GIVEN** gcc-4.0.4 builds successfully
- **WHEN** C and C++ smoke programs are compiled
- **THEN** the evidence proves the produced compiler output works without host fallback
