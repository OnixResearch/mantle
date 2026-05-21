## MODIFIED Requirements

### Requirement: GCC version ladder
GCC 4.0 source-frontier evidence SHALL remain partial and fail-closed unless native source-build correctness is proven.

#### Scenario: GCC 4.0 native cc1 c-parse make-log summary frontier is accepted

- GIVEN source-frontier evidence uses schema `mantle-gcc40-native-cc1-source-frontier-reduction-v9`
- AND the diagnostic derivation records compact make-log summary markers for compile-command presence, line count, and byte count
- AND the evidence preserves the focused `c-parse.o` make-error boundary and non-promotion semantics
- WHEN the bootstrap parity report evaluates `gcc.4.0`
- THEN the source-frontier evidence check passes
- AND `gcc.4.0` remains evidence-backed `partial` without completing live-bootstrap, Guix, or StageX parity

#### Scenario: GCC 4.0 native cc1 c-parse make-log summary frontier rejects stale evidence

- GIVEN the source-frontier evidence uses a stale schema, omits any make-log summary marker, omits the focused make-error boundary, or claims promotion
- WHEN the bootstrap parity report evaluates `gcc.4.0`
- THEN the row MUST remain a blocker
- AND the row notes the specific failed native source-frontier evidence check
