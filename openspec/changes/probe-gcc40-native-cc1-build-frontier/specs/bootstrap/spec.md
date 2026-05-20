## MODIFIED Requirements

### Requirement: GCC version ladder

GCC 4.0 native source-build frontier evidence MUST be checked separately from bounded installed-`cc1` semantic slices. A promoted native `cc1` build-frontier receipt MUST name exact derivation markers for the native make attempt, source-boundary diagnostic, pass1 bridge fallback, current source-frontier notes, and a retirement condition, and it MUST NOT claim full GCC 4.0 compiler correctness.

#### Scenario: GCC 4.0 native cc1 build-frontier receipt is accepted [r[bootstrap.gcc.version-ladder.gcc40-native-cc1-build-frontier]]

- GIVEN `bootstrap/gcc-4.0.ncl` contains exact markers for the native `cc1` source-build attempt, the TinyCC/Mes source-boundary diagnostic, and the pass1 bridge fallback
- AND a checked native-cc1 build-frontier receipt names the derivation, schema, required markers, source-frontier notes, partial-only parity effect, and retirement condition
- WHEN the bootstrap parity report evaluates the `gcc.4.0` row
- THEN the row may report evidence-backed `partial` with native source-build frontier evidence
- AND the row MUST continue blocking live-bootstrap, Guix, and StageX until full native GCC 4.0 compiler correctness exists

#### Scenario: GCC 4.0 native cc1 build-frontier receipt rejects stale evidence [r[bootstrap.gcc.version-ladder.gcc40-native-cc1-build-frontier-drift]]

- GIVEN the build-frontier receipt references missing derivation markers, stale source-frontier notes, an unsupported schema, missing retirement condition, or unsupported parity effect
- WHEN the bootstrap parity report evaluates the `gcc.4.0` row
- THEN the row MUST remain a blocker
- AND the row notes the specific failed native source-build frontier evidence check

#### Scenario: GCC 4.0 native cc1 build-frontier receipt cannot complete GCC 4.0 parity [r[bootstrap.gcc.version-ladder.gcc40-native-cc1-build-frontier-no-overclaim]]

- GIVEN native `cc1` build-frontier receipt evidence exists alongside bounded installed-`cc1` semantic slices
- WHEN live-bootstrap, Guix, or StageX parity is required
- THEN `gcc.4.0` MUST remain `partial` until full native compiler/generator/demangler correctness evidence is complete
- AND parity requirements still fail closed on the remaining GCC 4.0 blockers
