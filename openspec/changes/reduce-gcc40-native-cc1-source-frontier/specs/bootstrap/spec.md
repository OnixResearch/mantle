## MODIFIED Requirements

### Requirement: GCC version ladder

GCC 4.0 native `cc1` source-frontier reductions MUST be evidence-backed separately from bounded installed-`cc1` semantic slices. A promoted source-frontier reduction MUST name the prior frontier, the attempted bounded probe, exact derivation/source markers, the observed new frontier or unchanged blocker, and a retirement condition. It MUST NOT claim full GCC 4.0 compiler correctness unless separate complete native compiler/generator/demangler evidence exists.

#### Scenario: GCC 4.0 native cc1 source frontier reduction is accepted [r[bootstrap.gcc.version-ladder.gcc40-native-cc1-source-frontier-reduction]]

- GIVEN `bootstrap/gcc-4.0.ncl` contains exact markers for a bounded native `cc1` source-build probe near the current TinyCC/Mes frontier
- AND checked source-frontier evidence names the prior frontier, attempted command or patch scope, observed frontier result, exact markers, partial-only parity effect, and retirement condition
- WHEN the bootstrap parity report evaluates the `gcc.4.0` row
- THEN the row may report evidence-backed `partial` with updated native source-frontier evidence
- AND the row MUST continue blocking live-bootstrap, Guix, and StageX until full native GCC 4.0 compiler correctness exists

#### Scenario: GCC 4.0 native cc1 source frontier evidence rejects stale or missing results [r[bootstrap.gcc.version-ladder.gcc40-native-cc1-source-frontier-reduction-drift]]

- GIVEN the source-frontier evidence references missing derivation markers, omits the prior frontier, omits the observed frontier result, uses an unsupported schema/status, or lacks a retirement condition
- WHEN the bootstrap parity report evaluates the `gcc.4.0` row
- THEN the row MUST remain a blocker
- AND the row notes the specific failed native source-frontier evidence check

#### Scenario: GCC 4.0 native cc1 source frontier evidence cannot complete GCC 4.0 parity [r[bootstrap.gcc.version-ladder.gcc40-native-cc1-source-frontier-reduction-no-overclaim]]

- GIVEN native `cc1` source-frontier reduction evidence exists alongside bounded installed-`cc1` semantic slices and build-frontier metadata
- WHEN live-bootstrap, Guix, or StageX parity is required
- THEN `gcc.4.0` MUST remain `partial` until full native compiler/generator/demangler correctness evidence is complete
- AND parity requirements still fail closed on the remaining GCC 4.0 blockers
