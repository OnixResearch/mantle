## MODIFIED Requirements

### Requirement: GCC version ladder

GCC 4.0 native generator frontier reductions MUST be evidence-backed as bounded slices. A bounded `genrecog` slice MUST include source-bound derivation markers, recomputed digest evidence, stale-boundary rejection, and explicit non-claim wording so `gcc.4.0` remains partial until full native compiler and generator correctness is proven.

#### Scenario: GCC 4.0 native genrecog slice is accepted [r[bootstrap.gcc.version-ladder.gcc40-native-genrecog-slice]]

- GIVEN `bootstrap/gcc-4.0.ncl` contains a checked bounded `genrecog` output marker for GCC 4.0
- AND a checked native-generator receipt names `genrecog`, the derivation, schema version, bounded output contract, source marker, and digest/transcript evidence
- WHEN `bootstrap parity-report` evaluates the GCC 4.0 row
- THEN the row may report evidence-backed `partial` with native `genrecog` slice evidence
- AND the row MUST continue blocking live-bootstrap and Guix until full native GCC 4.0 compiler and generator correctness evidence exists

#### Scenario: GCC 4.0 native genrecog slice rejects stale boundary evidence [r[bootstrap.gcc.version-ladder.gcc40-native-genrecog-slice-drift]]

- GIVEN a native-generator receipt for `genrecog` references a missing marker, stale digest, unsupported schema, unsupported selected generator, missing bounded output contract, or unsupported parity effect
- WHEN the bootstrap parity report evaluates `gcc.4.0`
- THEN the row remains a blocker
- AND the row notes the specific failed native-generator evidence check

#### Scenario: GCC 4.0 native genrecog slice cannot complete GCC 4.0 parity [r[bootstrap.gcc.version-ladder.gcc40-native-genrecog-slice-no-overclaim]]

- GIVEN bounded generator member evidence exists for `genrecog`
- WHEN bootstrap parity claim gating evaluates live-bootstrap or Guix
- THEN `gcc.4.0` remains `partial`
- AND live-bootstrap and Guix requirements still fail closed until all remaining native compiler and generator correctness blockers have evidence
