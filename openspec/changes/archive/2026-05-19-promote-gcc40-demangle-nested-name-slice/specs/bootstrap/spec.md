## MODIFIED Requirements

### Requirement: GCC version ladder

GCC 4.0 pass1 libgcc member promotions MUST be evidence-backed one member at a time. The `__gcc_bcmp` member MUST implement byte-wise comparison semantics: it MUST return `0` for equal byte ranges and a non-zero value for the first differing byte over the requested length. The promotion MUST include a derivation-local smoke that exercises equal and unequal comparisons. GCC 4.0 pass1 driver query semantics MUST also be evidence-backed: `-dumpversion`, `-dumpmachine`, `-print-libgcc-file-name`, and `-print-search-dirs` MUST return deterministic GCC-shaped values tied to the installed artifact. GCC 4.0 installed `cc1` object-output semantics MUST be evidence-backed for the bounded GCC-shaped frontend invocation already accepted by the pass1 bridge. A promoted native `cc1` arithmetic slice MUST prove that the installed frontend handles a bounded C function containing integer arithmetic, comparison, branch, and return semantics without delegating object emission to TinyCC; it MUST record a checked receipt and smoke transcript, and it MUST NOT claim full GCC 4.0 compiler correctness. Promoted native generator-frontier slices MUST prove selected generator members' bounded output contracts with checked receipt evidence, while leaving unpromoted generator members and full native GCC 4.0 correctness blocked. Promoted native libiberty demangle slices MUST prove selected bounded Itanium demangle shapes with checked receipt evidence, while leaving full `cp-demangle` and native GCC 4.0 correctness blocked.

#### Scenario: GCC 4.0 native demangle nested-name slice is accepted [r[bootstrap.gcc.version-ladder.gcc40-native-demangle-nested-name-slice]]

- GIVEN `bootstrap/gcc-4.0.ncl` contains a checked bounded libiberty demangle semantic marker for GCC 4.0
- AND a checked native-demangle receipt names the selected nested-name shape, the derivation, schema version, bounded input/output contract, source markers, and digest or transcript evidence
- WHEN the bootstrap parity report evaluates `gcc.4.0`
- THEN the row may report evidence-backed `partial` with native demangle nested-name slice evidence
- AND the row MUST continue blocking live-bootstrap and Guix until full native GCC 4.0 compiler and demangler correctness evidence exists

#### Scenario: GCC 4.0 native demangle nested-name slice rejects stale boundary evidence [r[bootstrap.gcc.version-ladder.gcc40-native-demangle-nested-name-slice-drift]]

- GIVEN a native-demangle receipt references a missing marker, stale digest, unsupported schema, unsupported selected shape, missing bounded input/output contract, forbidden stale marker, or unsupported parity effect
- WHEN the bootstrap parity report evaluates `gcc.4.0`
- THEN the row remains a blocker
- AND the row notes the specific failed native-demangle evidence check

#### Scenario: GCC 4.0 native demangle nested-name slice cannot complete GCC 4.0 parity [r[bootstrap.gcc.version-ladder.gcc40-native-demangle-nested-name-slice-no-overclaim]]

- GIVEN bounded libiberty demangle evidence exists for the selected nested-name shape
- WHEN bootstrap parity claim gating evaluates live-bootstrap or Guix
- THEN `gcc.4.0` remains `partial`
- AND live-bootstrap and Guix requirements still fail closed until all remaining native compiler and demangler correctness blockers have evidence
