## MODIFIED Requirements

### Requirement: GCC version ladder

GCC 4.0 installed `cc1` native semantic slices MUST be evidence-backed one bounded input at a time. A promoted native logical/control-flow slice MUST prove that the installed frontend handles a selected C function containing comparison, logical `&&`/`||`, branch, and return behavior without delegating object emission to TinyCC; it MUST record checked receipt and transcript evidence, preserve the prior arithmetic/control-flow regression, and MUST NOT claim full GCC 4.0 compiler correctness.

#### Scenario: GCC 4.0 native cc1 logical/control-flow slice is accepted [r[bootstrap.gcc.version-ladder.gcc40-native-cc1-logical-slice]]

- GIVEN `bootstrap/gcc-4.0.ncl` contains a checked no-TinyCC-delegation marker for the selected bounded logical/control-flow input
- AND a checked native-cc1 receipt names schema `mantle-gcc40-native-cc1-arithmetic-v2`, selected slice `logical-boolean-control-flow-v2`, the derivation, bounded input, transcript, output digest, and preserved arithmetic regression
- WHEN the bootstrap parity report evaluates the `gcc.4.0` row
- THEN the row may report evidence-backed `partial` with native `cc1` logical/control-flow slice evidence
- AND the row MUST continue blocking live-bootstrap, Guix, and StageX until full native GCC 4.0 compiler correctness exists

#### Scenario: GCC 4.0 native cc1 logical/control-flow slice rejects stale or delegated evidence [r[bootstrap.gcc.version-ladder.gcc40-native-cc1-logical-slice-drift]]

- GIVEN the native-cc1 receipt references stale v1-only schema, omits the selected logical marker, has digest drift, contains TinyCC delegation markers in the selected transcript, or omits the arithmetic regression
- WHEN the bootstrap parity report evaluates the `gcc.4.0` row
- THEN the row MUST remain a blocker
- AND the row notes the specific failed native-cc1 evidence check

#### Scenario: GCC 4.0 native cc1 logical/control-flow slice cannot complete GCC 4.0 parity [r[bootstrap.gcc.version-ladder.gcc40-native-cc1-logical-slice-no-overclaim]]

- GIVEN bounded native `cc1` evidence exists for the selected arithmetic and logical/control-flow inputs
- WHEN live-bootstrap, Guix, or StageX parity is required
- THEN `gcc.4.0` MUST remain `partial` until full native compiler/generator/demangler correctness evidence is complete
- AND parity requirements still fail closed on the remaining GCC 4.0 blockers
