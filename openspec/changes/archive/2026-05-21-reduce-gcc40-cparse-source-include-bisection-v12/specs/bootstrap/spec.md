## MODIFIED Requirements

### Requirement: GCC version ladder

GCC 4.0 pass1 libgcc member promotions, native frontier receipts, and native source-frontier reductions MUST remain evidence-backed, fail closed on stale evidence, and MUST NOT complete live-bootstrap, Guix, or StageX parity until full native GCC 4.0 compiler correctness exists.

#### Scenario: GCC 4.0 native cc1 c-parse source include bisection frontier is accepted [r[bootstrap.gcc.version-ladder.gcc40-native-cc1-cparse-source-include-bisection-frontier]]

- GIVEN `bootstrap/evidence/gcc-4.0-native-cc1-build-frontier.json` uses source-frontier schema `mantle-gcc40-native-cc1-source-frontier-reduction-v12`
- AND the evidence preserves v11 include-flood truncation and filename-payload absence evidence
- AND the evidence records a compact source-level include-prefix bisection derived from `gcc/c-parse.c`'s ordered include list
- AND the evidence names exact diagnostic markers from `bootstrap/diag-gcc40-c-parse-boundary.ncl` for that source include-prefix boundary
- WHEN the bootstrap parity report evaluates the `gcc.4.0` row
- THEN the source-frontier evidence check passes
- AND `gcc.4.0` remains evidence-backed `partial` without completing live-bootstrap, Guix, or StageX parity

#### Scenario: GCC 4.0 native cc1 c-parse source include bisection frontier rejects stale evidence [r[bootstrap.gcc.version-ladder.gcc40-native-cc1-cparse-source-include-bisection-frontier-drift]]

- GIVEN the source-frontier evidence uses a stale schema, omits v11 truncation evidence, omits the source include-prefix boundary, claims the truncated make log identified a filename payload, or references diagnostic markers absent from the diagnostic derivation
- WHEN the bootstrap parity report evaluates the `gcc.4.0` row
- THEN the row MUST remain a blocker
- AND the row notes the specific failed native source-frontier evidence check
