## ADDED Requirements

### Requirement: Strict hermeticity regression suite

r[verification_evidence.strict_hermeticity_regression_suite] Mantle MUST maintain a strict hermeticity regression suite with a clean strict success fixture and negative fixtures for environment poisoning, PATH poisoning, undeclared host execution, network access, missing closure facts, umask drift, temp-root dependence, and nondeterministic output behavior.

#### Scenario: suite inventory declares expected outcomes

GIVEN the strict hermeticity regression suite is run
WHEN Mantle creates or reads the suite plan
THEN the plan MUST list each fixture, perturbation axis, expected verdict, required blocker or audit class, and unsupported-host non-claim behavior
AND the suite report MUST bind the plan digest.

#### Scenario: clean strict fixture succeeds

GIVEN the clean strict fixture has declared inputs, declared tools, no denied ambient environment, no network need, and complete closure facts
WHEN the regression suite executes that fixture
THEN the fixture MUST produce accepted strict hermeticity evidence and stable output digest evidence
AND the report MUST identify it as positive coverage only for the declared axes.

#### Scenario: poisoned fixtures fail closed

GIVEN a negative fixture introduces environment poisoning, PATH poisoning, undeclared host execution, network access, missing closure facts, umask drift, temp-root dependence, or nondeterministic output behavior
WHEN the regression suite executes that fixture under strict policy
THEN Mantle MUST fail closed or report the required blocker before admitting proof evidence
AND the suite report MUST NOT treat skipped or unsupported axes as passing coverage.
