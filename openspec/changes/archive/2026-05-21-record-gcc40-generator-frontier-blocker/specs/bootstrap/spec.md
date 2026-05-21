## MODIFIED Requirements

### Requirement: GCC version ladder

The bootstrap parity report SHALL keep GCC ladder rows fail-closed and evidence-backed, preserving partial status unless complete source/native correctness evidence exists.

#### Scenario: GCC 4.0 native generator frontier blocker is retained [r[bootstrap.gcc.version-ladder.gcc40-generator-frontier-blocker]]

- GIVEN the GCC 4.0 native boundary receipt records bounded `genattrtab` and `genoutput` generator output slices
- AND the receipt records a `generator-bounded-outputs` entry in `native_frontier.blockers`
- AND that blocker uses a source-bound derivation marker from `bootstrap/gcc-4.0.ncl`
- WHEN `bootstrap parity-report` evaluates the GCC 4.0 row
- THEN the row MUST remain evidence-backed partial
- AND the row MUST continue to report that full native generator correctness is pending

#### Scenario: GCC 4.0 native generator frontier blocker rejects drift [r[bootstrap.gcc.version-ladder.gcc40-generator-frontier-blocker-drift]]

- GIVEN the native boundary receipt omits the `generator-bounded-outputs` frontier blocker, references a missing generator derivation marker, or claims full native generator correctness
- WHEN `bootstrap parity-report` validates GCC 4.0 evidence
- THEN the row MUST remain a blocker
- AND the row notes the specific failed native-boundary evidence check
