## MODIFIED Requirements

### Requirement: GCC version ladder

The bootstrap parity report MUST keep GCC 4.0 native `cc1` source-frontier reductions evidence-backed and fail-closed without promoting full GCC 4.0 native correctness.

#### Scenario: c-parse sys/types preinclude make frontier records bounded v20 evidence [r[bootstrap.gcc.version-ladder.gcc40-cparse-systypes-config-frontier-v20]]

- GIVEN the v18 source-frontier receipt showed host `ssize_t` visibility before generated `config.h` makes compact stdio probes pass
- AND the v19 adjusted-config append-only `#undef ssize_t` real make probe did not advance the frontier
- WHEN the bootstrap parity report validates GCC 4.0 native `cc1` source-frontier evidence
- THEN the receipt uses schema `mantle-gcc40-native-cc1-source-frontier-reduction-v20`
- AND the observed frontier records the real `c-parse.o` make result after preincluding `<sys/types.h>` at the top of generated `gcc/config.h`
- AND the row remains `partial` and non-promoting.

#### Scenario: c-parse sys/types preinclude make frontier rejects stale evidence [r[bootstrap.gcc.version-ladder.gcc40-cparse-systypes-config-frontier-v20-drift]]

- GIVEN the source-frontier evidence uses a stale schema, omits the v19 adjusted-config marker, omits the v20 sys/types-preinclude make marker, or claims native GCC 4.0 correctness
- WHEN the parity report validates GCC 4.0
- THEN validation fails closed instead of promoting `gcc.4.0`.
