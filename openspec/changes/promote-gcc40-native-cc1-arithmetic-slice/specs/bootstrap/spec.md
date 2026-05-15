## MODIFIED Requirements

### Requirement: GCC version ladder

GCC 4.0 pass1 libgcc member promotions MUST be evidence-backed one member at a time. The `__gcc_bcmp` member MUST implement byte-wise comparison semantics: it MUST return `0` for equal byte ranges and a non-zero value for the first differing byte over the requested length. The promotion MUST include a derivation-local smoke that exercises equal and unequal comparisons. GCC 4.0 pass1 driver query semantics MUST also be evidence-backed: `-dumpversion`, `-dumpmachine`, `-print-libgcc-file-name`, and `-print-search-dirs` MUST return deterministic GCC-shaped values tied to the installed artifact. GCC 4.0 installed `cc1` object-output semantics MUST be evidence-backed for the bounded GCC-shaped frontend invocation already accepted by the pass1 bridge. A promoted native `cc1` arithmetic slice MUST prove that the installed frontend handles a bounded C function containing integer arithmetic, comparison, branch, and return semantics without delegating object emission to TinyCC; it MUST record a checked receipt and smoke transcript, and it MUST NOT claim full GCC 4.0 compiler correctness.

#### Scenario: GCC 4.0 native cc1 arithmetic slice is accepted [r[bootstrap.gcc.version-ladder.gcc40-native-cc1-arithmetic]]

- GIVEN `bootstrap/gcc-4.0.ncl` installs a `cc1` frontend for GCC 4.0
- AND a checked native-cc1 arithmetic receipt names the derivation, schema version, bounded input program, smoke command, and no-TinyCC-delegation evidence
- WHEN the bootstrap parity report evaluates `gcc.4.0`
- THEN the row may report evidence-backed `partial` with native-cc1 arithmetic slice evidence
- AND the row MUST continue blocking live-bootstrap and Guix until full native GCC 4.0 compiler and generator correctness evidence exists

#### Scenario: GCC 4.0 native cc1 arithmetic slice rejects TinyCC delegation [r[bootstrap.gcc.version-ladder.gcc40-native-cc1-arithmetic-no-tcc]]

- GIVEN the installed `cc1` still delegates object emission through TinyCC or the receipt cannot prove delegation absence for the bounded smoke
- WHEN the native-cc1 arithmetic receipt is validated
- THEN validation fails closed
- AND the parity row notes the remaining `cc1` delegation frontier

#### Scenario: GCC 4.0 native cc1 arithmetic receipt drift fails closed [r[bootstrap.gcc.version-ladder.gcc40-native-cc1-arithmetic-drift]]

- GIVEN a native-cc1 arithmetic receipt references a missing marker, stale transcript digest, unsupported schema, missing smoke input, or unsupported parity effect
- WHEN the bootstrap parity report evaluates `gcc.4.0`
- THEN the row remains a blocker
- AND the row notes the specific failed native-cc1 evidence check
