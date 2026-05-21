## MODIFIED Requirements

### Requirement: GCC version ladder

The GCC 4.0 native `cc1` source-frontier evidence MUST include a bounded v14 `stdio.h` frontier slice after v13 identified `<stdio.h>` as the first failing direct include from `system.h`, and MUST continue to report `gcc.4.0` as partial until a real native source-build/compiler-correctness proof exists.

#### Scenario: c-parse stdio frontier records bounded v14 evidence

- GIVEN the v13 source-frontier receipt narrowed the nested `system.h` include-prefix boundary to adding `<stdio.h>`
- WHEN the bootstrap parity report validates GCC 4.0 native `cc1` source-frontier evidence
- THEN the receipt uses schema `mantle-gcc40-native-cc1-source-frontier-reduction-v14`
- AND the observed frontier records compact `stdio.h` probe markers distinguishing the prior successful prefix from the first failing stdio-related shape
- AND the row remains `partial` and non-promoting.

#### Scenario: c-parse stdio frontier rejects stale evidence

- GIVEN the source-frontier evidence uses a stale schema, omits the v13 system-header boundary, omits the v14 stdio frontier markers, or claims native GCC 4.0 correctness
- WHEN the parity report validates GCC 4.0
- THEN validation fails closed instead of promoting `gcc.4.0`.
