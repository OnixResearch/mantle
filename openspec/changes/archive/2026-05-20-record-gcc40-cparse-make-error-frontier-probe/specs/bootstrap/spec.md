## MODIFIED Requirements

### Requirement: GCC version ladder

GCC 4.0 native `cc1` source-frontier evidence MUST accept a v7 c-parse make-error capture diagnostic reduction that records the focused `c-parse.o` make target's nonzero rc and bounded output markers after the v6 generated-header sweep. The evidence MUST remain source-frontier-only and MUST NOT complete `gcc.4.0` parity.

#### Scenario: GCC 4.0 native cc1 c-parse make-error frontier is accepted

- GIVEN `bootstrap/evidence/gcc-4.0-native-cc1-build-frontier.json` uses source-frontier schema `mantle-gcc40-native-cc1-source-frontier-reduction-v7`
- AND the receipt names exact diagnostic markers from `bootstrap/diag-gcc40-c-parse-boundary.ncl` for the captured `c-parse.o` make rc and bounded output head/tail
- WHEN the bootstrap parity report evaluates the `gcc.4.0` row
- THEN the source-frontier evidence check passes
- AND `gcc.4.0` remains evidence-backed `partial` without completing live-bootstrap, Guix, or StageX parity

#### Scenario: GCC 4.0 native cc1 c-parse make-error frontier rejects stale evidence

- GIVEN the source-frontier evidence uses a stale schema, omits the make-error capture diagnostic markers, omits the captured nonzero rc, or references diagnostic markers absent from the diagnostic derivation
- WHEN the bootstrap parity report evaluates the `gcc.4.0` row
- THEN the row MUST remain a blocker
- AND the row notes the specific failed native source-frontier evidence check
