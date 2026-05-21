## MODIFIED Requirements

### Requirement: GCC version ladder

The bootstrap parity model MUST keep GCC version rows evidence-backed and fail closed, preserving `partial` status for GCC 4.0 until bounded receipts are replaced by full native compiler/generator/demangler correctness evidence.

#### Scenario: GCC 4.0 native cc1 c-parse include-flood frontier is accepted [r[bootstrap.gcc.version-ladder.gcc40-native-cc1-cparse-include-flood-frontier]]

- GIVEN `bootstrap/evidence/gcc-4.0-native-cc1-build-frontier.json` uses source-frontier schema `mantle-gcc40-native-cc1-source-frontier-reduction-v10`
- AND the receipt preserves the focused `c-parse.o` make rc, compile-command, log-size, and tail markers from v9
- AND the diagnostic derivation emits compact include-flood presence and count markers from `/tmp/gcc40-cparse-make.log`
- WHEN bootstrap parity validates the GCC 4.0 row
- THEN the row may report the v10 include-flood frontier as evidence-backed partial
- AND `gcc.4.0` remains evidence-backed `partial` without completing live-bootstrap, Guix, or StageX parity

#### Scenario: GCC 4.0 native cc1 c-parse include-flood frontier rejects stale evidence [r[bootstrap.gcc.version-ladder.gcc40-native-cc1-cparse-include-flood-frontier-drift]]

- GIVEN the source-frontier evidence uses a stale schema, omits the v10 include-flood markers, omits the focused make-error boundary, or claims promotion
- WHEN bootstrap parity validates the GCC 4.0 row
- THEN the row remains a blocker
- AND the row notes the specific failed native source-frontier evidence check

#### Scenario: GCC 4.0 native cc1 c-parse include-flood frontier cannot complete GCC 4.0 parity [r[bootstrap.gcc.version-ladder.gcc40-native-cc1-cparse-include-flood-frontier-no-overclaim]]

- GIVEN v10 include-flood source-frontier evidence exists alongside bounded installed-`cc1`, demangle, and generator evidence
- WHEN live-bootstrap, Guix, or StageX parity is required
- THEN `gcc.4.0` MUST remain `partial` until full native compiler/generator/demangler correctness evidence is complete
- AND parity requirements still fail closed on the remaining GCC 4.0 blockers
