## MODIFIED Requirements

### Requirement: GCC version ladder

GCC 4.0 native `cc1` source-frontier evidence MUST accept a v6 c-parse full generated-header sweep diagnostic reduction that records the focused `c-parse.o` make attempt advancing beyond v5 representative generated-header prefix probes into later `machmode.h`, wider `tree.h`, and builtin enum probes while the real `c-parse.o` make target still fails at the full source-build boundary. The evidence MUST remain source-frontier-only and MUST NOT complete `gcc.4.0` parity.

#### Scenario: GCC 4.0 c-parse full generated-header frontier v6 is accepted

- GIVEN `bootstrap/evidence/gcc-4.0-native-cc1-build-frontier.json` uses source-frontier schema `mantle-gcc40-native-cc1-source-frontier-reduction-v6`
- AND the receipt names exact diagnostic markers from `bootstrap/diag-gcc40-c-parse-boundary.ncl` for later `machmode.h`, wider `tree.h`, builtin enum, and real `c-parse.o` make probes
- WHEN the bootstrap parity report evaluates `gcc.4.0`
- THEN the source-frontier evidence check passes
- AND `gcc.4.0` remains evidence-backed `partial` without completing live-bootstrap, Guix, or StageX parity

#### Scenario: GCC 4.0 c-parse full generated-header frontier v6 rejects stale evidence

- GIVEN the source-frontier evidence uses a stale schema, still claims the fdopen, autohost, or representative generated-header prefix seam is the active frontier, omits the v6 diagnostic markers, or references diagnostic markers absent from the diagnostic derivation
- WHEN the bootstrap parity report evaluates `gcc.4.0`
- THEN the row MUST remain a blocker
- AND the row notes the specific failed native source-frontier evidence check
