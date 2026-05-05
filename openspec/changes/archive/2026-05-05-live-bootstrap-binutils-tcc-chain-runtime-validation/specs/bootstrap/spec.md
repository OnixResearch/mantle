## ADDED Requirements

### Requirement: Binutils-tcc runtime validation completes independently [r[bootstrap.binutils.tcc.runtime-validation]]
The system MUST provide reproducible validation evidence for the binutils-tcc epoch chain when the local build runner requires more than one drain budget window.

#### Scenario: Longer-running Mes prerequisite [r[bootstrap.binutils.tcc.runtime-validation.mes-prerequisite]]
- **GIVEN** the binutils-tcc chain needs to build Mes before the first epoch root
- **WHEN** the validation is run with bubblewrap and writable local state/store directories
- **THEN** the evidence records whether Mes completed, failed, or required a separately cached prerequisite strategy

#### Scenario: Parent evidence set preserved [r[bootstrap.binutils.tcc.runtime-validation.parent-evidence]]
- **GIVEN** the parent V2-V5 validation tasks were deferred
- **WHEN** this follow-up completes
- **THEN** it records epoch builds, no-host leakage, post-musl linkage, and binutils smoke evidence sufficient to close the parent validation gap
