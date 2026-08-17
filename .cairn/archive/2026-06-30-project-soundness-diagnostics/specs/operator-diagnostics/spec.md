## ADDED Requirements

### Requirement: Project soundness diagnostics are stable and machine readable [r[operator_diagnostics.project_soundness_diagnostics]]

Mantle MUST render project soundness diagnostics with stable class identifiers, severity, subject, message, evidence references when available, and bounded fix guidance. JSON output MUST remain parseable and MUST include validity, issue count, highest severity, and ordered issues without mixing human diagnostics into stdout.

#### Scenario: JSON soundness output is deterministic [r[operator_diagnostics.project_soundness_diagnostics.scenario.json]]

- GIVEN equivalent project soundness issues are discovered in different traversal orders
- WHEN Mantle renders JSON diagnostics
- THEN Mantle MUST emit the same ordered issue list and summary fields
- AND stdout MUST contain only the documented JSON payload when `--json` is selected.

#### Scenario: Human diagnostics name failure class [r[operator_diagnostics.project_soundness_diagnostics.scenario.human]]

- GIVEN `mantle check` finds project soundness issues
- WHEN Mantle renders human output
- THEN each issue MUST include a stable class or concise class label, affected input/patch/file subject, and bounded fix guidance
- AND default output MUST remain free of verbose runtime fingerprints unless diagnostics are explicitly requested.

#### Scenario: Check does not overclaim readiness [r[operator_diagnostics.project_soundness_diagnostics.scenario.non-claim]]

- GIVEN project soundness diagnostics report no issues
- WHEN Mantle summarizes the result
- THEN the claim MUST be limited to manifest, lockfile, generated input, and configured project-state consistency
- AND it MUST NOT claim build success, source availability, trust satisfaction, or reproducibility unless separate evidence proves those facts.
