## MODIFIED Requirements

### Requirement: GCC version ladder

GCC 4.0 native source-frontier evidence MUST record bounded diagnostic seams without promoting `gcc.4.0` beyond evidence-backed `partial`.

#### Scenario: GCC 4.0 native cc1 source frontier decl0 probe is accepted [r[bootstrap.gcc.version-ladder.gcc40-cparse-decl0-frontier-probe]]

- GIVEN the native `cc1` build-frontier receipt records nested schema `mantle-gcc40-native-cc1-source-frontier-reduction-v2`
- AND the diagnostic derivation contains exact markers for `native387_disabled` compile-only success, musl-shim instrumented compiler selection, `cparse_decl0_trace_valid_var_semicolon rc=0`, and the remaining `fd_bad`/`tcc_write_elf_file` branch frontier
- WHEN the bootstrap parity report evaluates the `gcc.4.0` row
- THEN the row may report evidence-backed `partial` with the narrower c-parse/decl0 frontier evidence
- AND the row MUST continue blocking live-bootstrap, Guix, and StageX until full native GCC 4.0 compiler correctness exists

#### Scenario: GCC 4.0 native cc1 source frontier decl0 probe rejects stale evidence [r[bootstrap.gcc.version-ladder.gcc40-cparse-decl0-frontier-probe-drift]]

- GIVEN the v2 source-frontier receipt omits the diagnostic marker list, uses an older nested schema, records a non-frontier observed result, or references markers absent from the diagnostic derivation
- WHEN the bootstrap parity report evaluates the `gcc.4.0` row
- THEN the row MUST remain a blocker
- AND the row notes the specific failed source-frontier evidence check

#### Scenario: GCC 4.0 native cc1 source frontier decl0 probe cannot complete GCC 4.0 parity [r[bootstrap.gcc.version-ladder.gcc40-cparse-decl0-frontier-probe-no-overclaim]]

- GIVEN v2 source-frontier evidence exists alongside bounded installed-`cc1` semantic slices
- WHEN live-bootstrap, Guix, or StageX parity is required
- THEN `gcc.4.0` MUST remain `partial` until full native compiler/generator/demangler correctness evidence is complete
- AND the diagnostic probe MUST NOT be treated as native GCC 4.0 compiler correctness proof
