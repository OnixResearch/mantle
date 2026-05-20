## MODIFIED Requirements

### Requirement: GCC version ladder

GCC 4.0 native `cc1` source-frontier evidence MUST accept a v4 c-parse autohost macro-boundary diagnostic reduction that records the focused `c-parse.o` make attempt reaching the real c-parse compile, the bounded `auto-host.h` define window, and exact pass/fail probes around `NEED_64BIT_HOST_WIDE_INT`, `gid_t`, and `inline`. The evidence MUST remain source-frontier-only and MUST NOT complete `gcc.4.0` parity.

#### Scenario: GCC 4.0 c-parse autohost frontier v4 is accepted

- GIVEN `bootstrap/evidence/gcc-4.0-native-cc1-build-frontier.json` uses source-frontier schema `mantle-gcc40-native-cc1-source-frontier-reduction-v4`
- AND the receipt names exact diagnostic markers from `bootstrap/diag-gcc40-c-parse-boundary.ncl` for the autohost define window and the `NEED_64BIT_HOST_WIDE_INT`/`gid_t`/`inline` probe matrix
- WHEN the bootstrap parity report evaluates `gcc.4.0`
- THEN the source-frontier evidence check passes
- AND `gcc.4.0` remains evidence-backed `partial`
- AND live-bootstrap, Guix, and StageX parity remain blocked until native GCC 4.0 compiler correctness exists

#### Scenario: GCC 4.0 c-parse autohost frontier v4 rejects stale evidence

- GIVEN the receipt uses a stale schema, still claims the fdopen/output-return seam is the active remaining frontier, omits the autohost macro-boundary markers, or references diagnostic markers absent from the diagnostic derivation
- WHEN the bootstrap parity report evaluates `gcc.4.0`
- THEN the row MUST remain a blocker
- AND the row notes the specific failed native source-frontier evidence check
