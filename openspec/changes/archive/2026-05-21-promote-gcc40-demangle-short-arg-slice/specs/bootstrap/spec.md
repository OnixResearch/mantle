## MODIFIED Requirements

### Requirement: GCC version ladder

The bootstrap parity report SHALL keep GCC ladder rows fail-closed and evidence-backed, preserving partial status unless complete source/native correctness evidence exists.

#### Scenario: GCC 4.0 native demangle single-short Itanium slice is accepted [r[bootstrap.gcc.version-ladder.gcc40-native-demangle-short-arg]]

- GIVEN the GCC 4.0 native demangle receipt uses schema `mantle-gcc40-native-demangle-slice-v6`
- AND the receipt records `selected_shape=single-short-arg-itanium-v6`
- AND the bounded contract accepts `_ZN3foo3bar3bazEs -> foo::bar::baz(short)` while preserving previous zero-arg, single-int, single-char, and single-long regressions
- AND `bootstrap/gcc-4.0.ncl` contains the v6 cplus/cp-demangle source markers
- WHEN `bootstrap parity-report` evaluates the GCC 4.0 row
- THEN the row MUST remain evidence-backed partial
- AND the row MUST NOT claim native/full GCC 4.0 demangler correctness

#### Scenario: GCC 4.0 native demangle single-short slice rejects stale or overbroad evidence [r[bootstrap.gcc.version-ladder.gcc40-native-demangle-short-arg-drift]]

- GIVEN the demangle receipt uses a stale schema, omits the short-argument accepted input, omits the v6 source markers, reintroduces v5/v4/v3/v2/v1/v0 stale markers, or claims full native demangler correctness
- WHEN `bootstrap parity-report` validates GCC 4.0 evidence
- THEN the row MUST remain a blocker
- AND the row notes the specific failed demangle evidence check
