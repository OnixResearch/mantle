## ADDED Requirements

### Requirement: Nickel export diagnostics are stable and machine readable [r[operator_diagnostics.nickel_export_diagnostics]]

Mantle MUST render Nickel export diagnostics with stable schema labels, status fields, format, output target, receipt or digest references, and deterministic failure classes. JSON output MUST remain parseable and MUST NOT mix human diagnostics into stdout.

#### Scenario: JSON export output is parseable [r[operator_diagnostics.nickel_export_diagnostics.scenario.json]]

- GIVEN an operator runs a Nickel export command with `--json`
- WHEN the command succeeds or fails with a documented JSON result
- THEN stdout MUST contain only the documented export JSON payload
- AND human diagnostics, verbose fingerprints, and evaluator logs MUST be emitted outside stdout or captured as bounded fields.

#### Scenario: Human diagnostics name export failure class [r[operator_diagnostics.nickel_export_diagnostics.scenario.human-failure]]

- GIVEN a Nickel export request fails validation or evaluation
- WHEN Mantle renders human diagnostics
- THEN the diagnostic MUST include a stable failure class, affected source or import subject when safe to reveal, and bounded fix guidance
- AND it MUST NOT claim build success, deployability, or correctness beyond the failed export attempt.

#### Scenario: Export success remains a bounded claim [r[operator_diagnostics.nickel_export_diagnostics.scenario.non-claim]]

- GIVEN a Nickel export command succeeds
- WHEN Mantle summarizes the result
- THEN the summary MAY claim that the declared Nickel export produced the recorded digest under the recorded evaluator descriptor
- AND it MUST NOT claim that generated config is deployable, semantically valid for a frontend, or sufficient for a build unless separate evidence proves those facts.
