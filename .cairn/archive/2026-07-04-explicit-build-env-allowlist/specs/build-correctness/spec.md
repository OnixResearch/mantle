## ADDED Requirements

### Requirement: Explicit build environment allowlist

r[build_correctness.explicit_environment_allowlist] Mantle MUST construct strict build environments from declared environment entries and reviewed deterministic defaults, reject denied ambient variables, and bind the normalized environment digest into action and build evidence.

#### Scenario: declared environment is stable

GIVEN a strict build action declares environment entries and deterministic defaults
WHEN Mantle normalizes the build environment
THEN the child environment MUST contain only declared or policy-default entries
AND the action or build receipt MUST bind the normalized environment digest.

#### Scenario: ambient poisoning is rejected

GIVEN the parent process contains denied ambient variables such as dynamic-linker controls, compiler wrappers, proxy settings, token-like names, or undeclared locale overrides
WHEN Mantle prepares a strict build environment
THEN Mantle MUST reject or omit those variables before child execution according to policy
AND diagnostics MUST identify the denied class without copying secret values into reports.

#### Scenario: missing required variable fails closed

GIVEN a strict action requires a modeled environment entry that cannot be constructed from declared inputs or deterministic defaults
WHEN environment normalization runs
THEN Mantle MUST fail before execution with a deterministic missing-environment diagnostic
AND it MUST NOT inherit the missing value from the host environment.
