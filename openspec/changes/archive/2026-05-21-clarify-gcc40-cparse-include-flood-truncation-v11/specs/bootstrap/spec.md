## MODIFIED Requirements

### Requirement: GCC version ladder

GCC 4.0 native `cc1` source-frontier evidence MUST distinguish bounded diagnostic clarification from native compiler/source-build correctness.

#### Scenario: GCC 4.0 native cc1 c-parse include-flood truncation frontier is accepted

- GIVEN `bootstrap/evidence/gcc-4.0-native-cc1-build-frontier.json` uses source-frontier schema `mantle-gcc40-native-cc1-source-frontier-reduction-v11`
- AND the evidence preserves the v10 focused `c-parse.o` make rc, compile-command, log summary, include-flood count, bounded tail, and make-error evidence
- AND the evidence records that the two include-flood lines are truncated and carry no filename payload
- WHEN the bootstrap parity report evaluates the `gcc.4.0` row
- THEN the source-frontier evidence check passes
- AND `gcc.4.0` remains evidence-backed `partial` without completing live-bootstrap, Guix, or StageX parity

#### Scenario: GCC 4.0 native cc1 c-parse include-flood truncation frontier rejects stale or overclaiming evidence

- GIVEN the source-frontier evidence uses a stale schema, omits the truncation markers, omits filename-payload absence, omits the focused make failure evidence, or claims promotion
- WHEN the bootstrap parity report evaluates the `gcc.4.0` row
- THEN the row MUST remain a blocker
- AND the row notes the specific failed native source-frontier evidence check
