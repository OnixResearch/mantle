## MODIFIED Requirements

### Requirement: GCC version ladder

The GCC 4.0 native `cc1` source-frontier evidence MUST include a bounded v15 config/stdio interaction slice after v14 identified that `<stdio.h>` succeeds alone while `config.h` plus `<stdio.h>` fails and `config-undef6.h` plus `<stdio.h>` succeeds. The evidence MUST continue to report `gcc.4.0` as partial until a real native source-build/compiler-correctness proof exists.

#### Scenario: c-parse config/stdio interaction records bounded v15 evidence

- GIVEN the v14 source-frontier receipt narrowed the failure to the `config.h` plus `<stdio.h>` interaction
- WHEN the bootstrap parity report validates GCC 4.0 native `cc1` source-frontier evidence
- THEN the receipt uses schema `mantle-gcc40-native-cc1-source-frontier-reduction-v15`
- AND the observed frontier records compact config-fragment stdio probe markers that distinguish full `config.h` from the passing six-undef variant
- AND the row remains `partial` and non-promoting.

#### Scenario: c-parse config/stdio interaction rejects stale evidence

- GIVEN the source-frontier evidence uses a stale schema, omits the v14 stdio boundary, omits the v15 config/stdio interaction markers, or claims native GCC 4.0 correctness
- WHEN the parity report validates GCC 4.0
- THEN validation fails closed instead of promoting `gcc.4.0`.
