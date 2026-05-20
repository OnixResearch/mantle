## MODIFIED Requirements

### Requirement: GCC version ladder

GCC 4.0 installed `cc1` native semantic slices MUST be evidence-backed one bounded input at a time. A promoted native array-index slice MUST prove that the installed frontend handles a selected C input containing a local fixed `int` array declaration, indexed stores, indexed loads, expression use, and return behavior without delegating object emission to TinyCC; it MUST record checked receipt and transcript evidence, preserve the prior arithmetic/control-flow, logical/control-flow, local-variable, and helper-call regressions, and MUST NOT claim full GCC 4.0 compiler correctness.

#### Scenario: GCC 4.0 native cc1 array-index slice is accepted [r[bootstrap.gcc.version-ladder.gcc40-native-cc1-array-slice]]

- GIVEN `bootstrap/gcc-4.0.ncl` contains a checked no-TinyCC-delegation marker for the selected bounded array-index input
- AND a checked native-cc1 receipt names the new schema, selected slice `array-index-v5`, the derivation, bounded input, transcript, output digest, and preserved arithmetic, logical, local-variable, and helper-call regressions
- WHEN the bootstrap parity report evaluates the `gcc.4.0` row
- THEN the row may report evidence-backed `partial` with native `cc1` array-index slice evidence
- AND the row MUST continue blocking live-bootstrap, Guix, and StageX until full native GCC 4.0 compiler correctness exists

#### Scenario: GCC 4.0 native cc1 array-index slice rejects stale or delegated evidence [r[bootstrap.gcc.version-ladder.gcc40-native-cc1-array-slice-drift]]

- GIVEN the native-cc1 receipt references an older schema, omits the selected array-index marker, has digest drift, contains TinyCC delegation markers in the selected transcript, or omits any preserved regression
- WHEN the bootstrap parity report evaluates the `gcc.4.0` row
- THEN the row MUST remain a blocker
- AND the row notes the specific failed native-cc1 evidence check

#### Scenario: GCC 4.0 native cc1 array-index slice cannot complete GCC 4.0 parity [r[bootstrap.gcc.version-ladder.gcc40-native-cc1-array-slice-no-overclaim]]

- GIVEN bounded native `cc1` evidence exists for the selected arithmetic, logical/control-flow, local-variable assignment, helper-call, and array-index inputs
- WHEN live-bootstrap, Guix, or StageX parity is required
- THEN `gcc.4.0` MUST remain `partial` until full native compiler/generator/demangler correctness evidence is complete
- AND parity requirements still fail closed on the remaining GCC 4.0 blockers
