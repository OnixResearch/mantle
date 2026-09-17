# Specification: Long-lived service readiness

## ADDED Requirements

### Requirement: One readiness vocabulary

r[mantle.service_readiness.readiness_vocabulary] Long-lived Mantle components
MUST report readiness through one declared vocabulary: `started`, `ready`,
`complete`, `failed`, plus declared user-defined states. A component MUST be
able to report `started` and `ready` separately, and `ready` MUST mean the
component can take work.

The coordination daemon from ADR 0080 MUST report its own readiness under the
same vocabulary.

An unknown state MUST be rejected at admission.

#### Scenario: Coordination daemon reports readiness

- GIVEN the coordination daemon starting with no published facts
- WHEN it starts and can serve subscriptions
- THEN it MUST report `started` and `ready` separately
- AND a restart MUST report `started` again before it reports `ready`

#### Scenario: Ready follows a successful request

- GIVEN a long-lived component that has started but not finished initializing
- WHEN the component finishes initializing and accepts one real request
- THEN it MUST report `started` and `ready`
- AND before that point it MUST NOT report `ready`

#### Scenario: Early exit reports failure

- GIVEN a component process that exits before announcing readiness
- WHEN the supervisor observes the exit
- THEN the component MUST report `failed`
- AND it MUST NOT report `ready`

### Requirement: Dependencies are declared and block readiness

r[mantle.service_readiness.declared_dependencies] Each long-lived component and
declared proof stage MUST declare the components it depends on. A component
MUST NOT report `ready` before every declared dependency reports `ready` or
`complete`. A blocked component MUST report the blocking dependency.

#### Scenario: Dependent waits for its dependency

- GIVEN component `b` declares a dependency on component `a`
- AND `a` has started but is not ready
- WHEN `b` evaluates its readiness
- THEN `b` MUST report not-ready with `a` named as the blocker

### Requirement: Restart policy comes from a closed matrix

r[mantle.service_readiness.restart_policy_matrix] Every supervised component
MUST name exactly one restart policy from `always`, `on-error`, `all`, and
`never`. The declared policy MUST appear in the component's runtime report. A
missing or unknown policy MUST be rejected.

#### Scenario: Policy controls restart behavior

- GIVEN a component with restart policy `never`
- WHEN the component exits abnormally
- THEN the supervisor MUST NOT restart it
- AND the component MUST report a terminal state

### Requirement: Doctor publishes derived readiness state

r[mantle.service_readiness.doctor_derived_state] `mantle doctor` MUST be able
to publish derived readiness state for the declared components. The published
state MUST be marked as coordination state, MUST NOT be accepted as evidence,
and MUST NOT change existing doctor human or JSON output shapes.

#### Scenario: Doctor state is not evidence

- GIVEN a published doctor readiness state
- WHEN an evidence validator or receipt builder reads it
- THEN it MUST be rejected as evidence
- AND the same doctor run MUST produce its existing output shape
