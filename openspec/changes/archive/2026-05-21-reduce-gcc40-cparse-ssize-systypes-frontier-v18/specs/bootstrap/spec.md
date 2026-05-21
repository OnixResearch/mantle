## MODIFIED Requirements

### Requirement: GCC version ladder

The bootstrap parity report MUST keep GCC 4.0 native `cc1` source-frontier reductions evidence-backed and fail-closed without promoting full GCC 4.0 native correctness.

#### Scenario: c-parse ssize_t sys/types preinclude frontier records bounded v18 evidence [r[bootstrap.gcc.version-ladder.gcc40-cparse-ssize-systypes-frontier-v18]]

- GIVEN the v17 source-frontier receipt showed that both pre-stdio macro and typedef `ssize_t` definitions reproduce the include-flood failure
- WHEN the bootstrap parity report validates GCC 4.0 native `cc1` source-frontier evidence
- THEN the receipt uses schema `mantle-gcc40-native-cc1-source-frontier-reduction-v18`
- AND the observed frontier records compact `<sys/types.h>` before `config.h` markers that distinguish host typedef visibility from a bare pre-stdio `ssize_t` definition
- AND the row remains `partial` and non-promoting.

#### Scenario: c-parse ssize_t sys/types preinclude frontier rejects stale evidence [r[bootstrap.gcc.version-ladder.gcc40-cparse-ssize-systypes-frontier-v18-drift]]

- GIVEN the source-frontier evidence uses a stale schema, omits the v17 macro-vs-typedef markers, omits the v18 sys/types preinclude marker, or claims native GCC 4.0 correctness
- WHEN the parity report validates GCC 4.0
- THEN validation fails closed instead of promoting `gcc.4.0`.
