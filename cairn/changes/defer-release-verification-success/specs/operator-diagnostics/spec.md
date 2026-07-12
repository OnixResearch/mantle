# Operator Diagnostics Specification

## Purpose

Make human and JSON release verification output accurately represent the final policy verdict.

## Requirements

### Requirement: Release verification success is rendered only after acceptance

r[mantle.operator_diagnostics.release_verification.terminal_verdict] Mantle MUST render the human release verification success marker only when the completed final decision is valid, and MUST render explicit rejection wording without any success marker when selected policy fails.

#### Scenario: Human rejection cannot look successful

r[mantle.operator_diagnostics.release_verification.fixtures.negative]
- GIVEN an invocation passes manifest integrity but fails a required policy check
- WHEN human output is rendered
- THEN the process MUST exit nonzero and identify the rejection
- AND stdout and stderr MUST NOT contain `release evidence verified` or equivalent success wording.

#### Scenario: Human acceptance is terminal

r[mantle.operator_diagnostics.release_verification.fixtures.positive]
- GIVEN the completed final decision is valid
- WHEN human output is rendered
- THEN the success marker MUST appear only after every selected check is represented
- AND no later policy error MAY reverse that rendered verdict.

### Requirement: JSON release verification carries final validity

r[mantle.operator_diagnostics.release_verification.json_contract] Mantle JSON release verification output MUST include versioned top-level `valid`, closed `disposition`, ordered `checks`, and ordered `diagnostics` fields for accepted and policy-rejected decisions, with stdout containing exactly one JSON value.

#### Scenario: JSON rejection is parseable and non-promoting

r[mantle.operator_diagnostics.release_verification.json_negative]
- GIVEN a selected release policy is unsatisfied after manifest loading succeeds
- WHEN `mantle --json release verify` renders the final decision
- THEN stdout MUST contain one parseable result with `valid` set to false and a rejection disposition
- AND the process MUST exit nonzero without emitting a separate success payload or human text on stdout.

### Requirement: Verification rendering consumes the final decision

r[mantle.operator_diagnostics.release_verification.render_boundary] Human and JSON renderers MUST consume the immutable completed verification decision and MUST NOT perform or defer policy validation while rendering.

#### Scenario: Rendering cannot change validity

r[mantle.operator_diagnostics.release_verification.render_boundary.test]
- GIVEN an immutable accepted or rejected decision
- WHEN either renderer formats it
- THEN rendering MUST preserve validity, disposition, check ordering, and diagnostics
- AND it MUST have no authority to run a missing gate or convert a rejection into success.
