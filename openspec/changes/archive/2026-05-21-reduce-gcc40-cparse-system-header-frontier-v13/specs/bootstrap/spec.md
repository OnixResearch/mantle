## MODIFIED Requirements

### Requirement: GCC version ladder

The bootstrap parity evidence MUST preserve GCC 4.0 as partial while native source-frontier diagnostics are being reduced.

#### Scenario: GCC 4.0 c-parse system.h nested frontier is recorded
- GIVEN v12 narrowed the c-parse include-prefix seam to `config.h` plus `system.h`
- WHEN the GCC 4.0 c-parse source-frontier receipt is validated
- THEN it records ordered direct `system.h` include-prefix evidence
- AND it preserves the v12 source include-prefix evidence
- AND it preserves the v11 include-flood truncation evidence
- AND it states this is diagnostic/source-frontier evidence only

#### Scenario: Stale v12 c-parse source frontier is rejected
- GIVEN the c-parse source-frontier receipt lacks the v13 `system.h` nested-prefix evidence
- WHEN bootstrap parity evidence is evaluated
- THEN the evidence check fails closed instead of accepting stale v12-only evidence
