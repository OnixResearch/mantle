## MODIFIED Requirements

### Requirement: GCC version ladder

Mantle MUST keep GCC 4.0 native `cc1` c-parse source-frontier evidence bounded, schema-versioned, and fail-closed.

#### Scenario: c-parse ssize_t/stdio interaction records bounded v16 evidence [r[bootstrap.gcc.version-ladder.gcc40-cparse-ssize-stdio-frontier-v16]]

- GIVEN the v15 source-frontier receipt narrowed the `config.h` plus `<stdio.h>` failure to the single `ssize_t` undef probe
- WHEN the bootstrap parity report validates GCC 4.0 native `cc1` source-frontier evidence
- THEN the receipt uses schema `mantle-gcc40-native-cc1-source-frontier-reduction-v16`
- AND the observed frontier records compact `ssize_t` definition/order markers preserving the v15 config/stdio results
- AND the row remains `partial` and non-promoting.

#### Scenario: c-parse ssize_t/stdio interaction rejects stale evidence [r[bootstrap.gcc.version-ladder.gcc40-cparse-ssize-stdio-frontier-v16-drift]]

- GIVEN the source-frontier evidence uses a stale schema, omits the v15 config/stdio interaction markers, omits the v16 `ssize_t` definition/order markers, or claims native GCC 4.0 correctness
- WHEN the parity report validates GCC 4.0
- THEN validation fails closed instead of promoting `gcc.4.0`.
