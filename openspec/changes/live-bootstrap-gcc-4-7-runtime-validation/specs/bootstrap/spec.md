## ADDED Requirements

### Requirement: gcc-4.7 runtime validation completes after prerequisite stages [r[bootstrap.gcc47.runtime-validation]]
The system MUST validate gcc-4.7.4 only after binutils-tcc and gcc-4.0 runtime evidence is available or explicitly blocked.

#### Scenario: Build transcript records gcc-4.7 transition evidence [r[bootstrap.gcc47.runtime-validation.transcript]]
- **GIVEN** prerequisite runtime outputs are available
- **WHEN** `bootstrap/gcc-4.7.ncl` is built
- **THEN** the transcript records command, provider, exit status, output path or failure class, fallback status, and placeholder rejection

#### Scenario: C++11 smoke proves transition [r[bootstrap.gcc47.runtime-validation.smoke]]
- **GIVEN** gcc-4.7.4 builds successfully
- **WHEN** C, C++, and minimal C++11 smoke programs are compiled
- **THEN** the evidence proves the produced compiler output works without host fallback
