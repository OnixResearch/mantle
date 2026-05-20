## MODIFIED Requirements

### Requirement: GCC version ladder

GCC 4.0 installed `cc1` native semantic slices MUST be evidence-backed one bounded input at a time. A promoted native local-variable assignment slice MUST prove that the installed frontend handles a selected C function containing local `int` declarations, assignment or reassignment, expression use of those locals, and return behavior without delegating object emission to TinyCC; it MUST record checked receipt and transcript evidence, preserve the prior arithmetic/control-flow and logical/control-flow regressions, and MUST NOT claim full GCC 4.0 compiler correctness.

#### Scenario: GCC 4.0 native cc1 local-variable assignment slice is accepted [r[bootstrap.gcc.version-ladder.gcc40-native-cc1-local-vars-slice]]

- GIVEN `bootstrap/gcc-4.0.ncl` contains a checked no-TinyCC-delegation marker for the selected bounded local-variable assignment input
- AND a checked native-cc1 receipt names the new schema, selected slice `local-variable-assignment-v3`, the derivation, bounded input, transcript, output digest, and preserved arithmetic and logical regressions
- WHEN the bootstrap parity report evaluates the `gcc.4.0` row
- THEN the row may report evidence-backed `partial` with native `cc1` local-variable assignment slice evidence
- AND the row MUST continue blocking live-bootstrap, Guix, and StageX until full native GCC 4.0 compiler correctness exists

#### Scenario: GCC 4.0 native cc1 local-variable assignment slice rejects stale or delegated evidence [r[bootstrap.gcc.version-ladder.gcc40-native-cc1-local-vars-slice-drift]]

- GIVEN the native-cc1 receipt references an older schema, omits the selected local-variable marker, has digest drift, contains TinyCC delegation markers in the selected transcript, or omits either preserved regression
- WHEN the bootstrap parity report evaluates the `gcc.4.0` row
- THEN the row MUST remain a blocker
- AND the row notes the specific failed native-cc1 evidence check

#### Scenario: GCC 4.0 native cc1 local-variable assignment slice cannot complete GCC 4.0 parity [r[bootstrap.gcc.version-ladder.gcc40-native-cc1-local-vars-slice-no-overclaim]]

- GIVEN bounded native `cc1` evidence exists for the selected arithmetic, logical/control-flow, and local-variable assignment inputs
- WHEN live-bootstrap, Guix, or StageX parity is required
- THEN `gcc.4.0` MUST remain `partial` until full native compiler/generator/demangler correctness evidence is complete
- AND parity requirements still fail closed on the remaining GCC 4.0 blockers
