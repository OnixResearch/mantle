## MODIFIED Requirements

### Requirement: GCC version ladder

GCC 4.0 native `cc1` source-frontier evidence MUST accept a v5 c-parse generated-header diagnostic reduction that records the focused `c-parse.o` make attempt advancing beyond the v4 `auto-host.h` macro seam with six targeted undefines, then narrowing the next boundary to `insn-modes.h`, `machmode.h`, and early `tree.h` generated-header prefix probes. The evidence MUST remain source-frontier-only and MUST NOT complete `gcc.4.0` parity.

#### Scenario: GCC 4.0 c-parse generated-header frontier v5 is accepted

- GIVEN `bootstrap/evidence/gcc-4.0-native-cc1-build-frontier.json` uses source-frontier schema `mantle-gcc40-native-cc1-source-frontier-reduction-v5`
- AND the receipt names exact diagnostic markers from `bootstrap/diag-gcc40-c-parse-boundary.ncl` for the six-undef autohost probes and representative `insn-modes.h`, `machmode.h`, and `tree.h` generated-header prefix outcomes
- WHEN the bootstrap parity report evaluates `gcc.4.0`
- THEN the source-frontier evidence check passes
- AND `gcc.4.0` remains evidence-backed `partial`
- AND live-bootstrap, Guix, and StageX parity remain blocked until native GCC 4.0 compiler correctness exists

#### Scenario: GCC 4.0 c-parse generated-header frontier v5 rejects stale evidence

- GIVEN the receipt uses a stale schema, still claims the autohost macro seam is the active remaining frontier, omits generated-header boundary markers, or references diagnostic markers absent from the diagnostic derivation
- WHEN the bootstrap parity report evaluates `gcc.4.0`
- THEN the row MUST remain a blocker
- AND the row notes the specific failed native source-frontier evidence check
