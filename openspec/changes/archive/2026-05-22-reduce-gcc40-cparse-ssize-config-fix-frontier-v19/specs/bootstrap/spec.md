## MODIFIED Requirements

### Requirement: GCC version ladder

The bootstrap parity report MUST keep GCC 4.0 native `cc1` source-frontier reductions evidence-backed and fail-closed without promoting full GCC 4.0 native correctness.

#### Scenario: c-parse ssize_t adjusted-config make frontier records bounded v19 evidence [r[bootstrap.gcc.version-ladder.gcc40-cparse-ssize-config-fix-frontier-v19]]

- GIVEN the v18 source-frontier receipt showed host `ssize_t` visibility before generated `config.h` makes compact stdio probes pass
- WHEN the bootstrap parity report validates GCC 4.0 native `cc1` source-frontier evidence
- THEN the receipt uses schema `mantle-gcc40-native-cc1-source-frontier-reduction-v19`
- AND the observed frontier records the real `c-parse.o` make result after appending `#undef ssize_t` to generated `config.h`
- AND the row remains `partial` and non-promoting.

#### Scenario: c-parse ssize_t adjusted-config make frontier rejects stale evidence [r[bootstrap.gcc.version-ladder.gcc40-cparse-ssize-config-fix-frontier-v19-drift]]

- GIVEN the source-frontier evidence uses a stale schema, omits the v18 sys/types marker, omits the v19 adjusted-config make marker, or claims native GCC 4.0 correctness
- WHEN the parity report validates GCC 4.0
- THEN validation fails closed instead of promoting `gcc.4.0`.
