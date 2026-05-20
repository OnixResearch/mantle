## MODIFIED Requirements

### Requirement: GCC version ladder

GCC 4.0 native `cc1` source-frontier evidence MUST accept a v3 c-parse/decl0 diagnostic reduction that records the copied `fd_bad` branch being forced false and the bounded probe reaching `fdopen`, ELF output-format, and `tcc_output_file` return markers under the musl-shim instrumented compiler. The evidence MUST remain source-frontier-only and MUST NOT complete `gcc.4.0` parity.

#### Scenario: GCC 4.0 c-parse fdopen frontier v3 is accepted

- GIVEN `bootstrap/evidence/gcc-4.0-native-cc1-build-frontier.json` uses source-frontier schema `mantle-gcc40-native-cc1-source-frontier-reduction-v3`
- AND the receipt names exact diagnostic markers from `bootstrap/diag-gcc40-c-parse-boundary.ncl` for the copied `fd_bad` bypass and subsequent `fdopen`/output-return markers
- WHEN the bootstrap parity report evaluates `gcc.4.0`
- THEN the source-frontier evidence check passes
- AND `gcc.4.0` remains evidence-backed `partial`
- AND live-bootstrap, Guix, and StageX parity remain blocked until native GCC 4.0 compiler correctness exists

#### Scenario: GCC 4.0 c-parse fdopen frontier v3 rejects stale evidence

- GIVEN the receipt uses a stale schema, still claims the copied `fd_bad` branch is the active remaining frontier, omits the fdopen/output-return diagnostic markers, or references diagnostic markers absent from the diagnostic derivation
- WHEN the bootstrap parity report evaluates `gcc.4.0`
- THEN the row remains a blocker
- AND the failed source-frontier evidence check explains the stale or missing marker contract
