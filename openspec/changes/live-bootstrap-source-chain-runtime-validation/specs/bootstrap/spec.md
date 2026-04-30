## ADDED Requirements

### Requirement: Source-chain runtime validation completes after transition evidence [r[bootstrap.source.chain.runtime-validation]]
The system MUST keep full-source bootstrap proof incomplete until transition-build, final-provider, and self-build proof evidence is available.

#### Scenario: Transition blockers remain explicit [r[bootstrap.source.chain.runtime-validation.blockers]]
- **GIVEN** binutils-tcc or gcc transition validation is deferred
- **WHEN** source-chain status is reported
- **THEN** the evidence identifies the deferred follow-up changes and does not promote full-source status

#### Scenario: Final provider proof closes umbrella validation [r[bootstrap.source.chain.runtime-validation.final-proof]]
- **GIVEN** transition evidence is complete
- **WHEN** final provider and self-build proof validation runs
- **THEN** the evidence records normalized provider contract fields, fallback-event markers, and proof digests required for promotion
