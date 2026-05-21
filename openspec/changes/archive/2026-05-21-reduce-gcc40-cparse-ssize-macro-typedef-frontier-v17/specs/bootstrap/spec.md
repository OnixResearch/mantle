## MODIFIED Requirements

### Requirement: GCC version ladder

GCC 4.0 native `cc1` source-frontier evidence MUST remain fail-closed and MUST NOT promote the `gcc.4.0` parity row until native compiler correctness is proven.

#### Scenario: c-parse ssize_t macro-vs-typedef frontier records bounded v17 evidence [r[bootstrap.gcc.version-ladder.gcc40-cparse-ssize-macro-typedef-frontier-v17]]

- GIVEN the v16 source-frontier receipt showed that pre-stdio `#define ssize_t int` reproduces the include-flood failure while post-stdio redefinition succeeds
- WHEN the bootstrap parity report validates GCC 4.0 native `cc1` source-frontier evidence
- THEN the receipt uses schema `mantle-gcc40-native-cc1-source-frontier-reduction-v17`
- AND the observed frontier records compact `ssize_t` macro-vs-typedef markers showing both pre-stdio macro and typedef forms reproduce the include-flood failure while post-stdio redefinition still succeeds
- AND the row remains `partial` and non-promoting.

#### Scenario: c-parse ssize_t macro-vs-typedef frontier rejects stale evidence [r[bootstrap.gcc.version-ladder.gcc40-cparse-ssize-macro-typedef-frontier-v17-drift]]

- GIVEN the source-frontier evidence uses a stale schema, omits the v16 `ssize_t` order markers, omits the v17 macro-vs-typedef markers, or claims native GCC 4.0 correctness
- WHEN the parity report validates GCC 4.0
- THEN validation fails closed instead of promoting `gcc.4.0`.
