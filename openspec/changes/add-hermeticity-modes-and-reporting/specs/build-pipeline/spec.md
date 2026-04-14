## ADDED Requirements

### Requirement: Build pipeline carries explicit hermeticity mode

The build pipeline MUST accept and preserve an explicit hermeticity mode for
all build-entry runs.

At minimum the pipeline MUST distinguish between `practical` and `strict`
execution modes and make that selection available to build-finalization and
reporting code.

#### Scenario: Pipeline receives strict mode unchanged

- GIVEN a build-entry command selected hermeticity mode `strict`
- WHEN the pipeline starts the build run
- THEN the pipeline retains that exact mode selection
- AND later build stages can branch on `strict` without guessing from CLI flags

### Requirement: Build pipeline records typed hermeticity audit events

The build pipeline MUST record degraded execution facts as typed hermeticity
audit events.

Those events MUST be attached to the build result even when the build succeeds.
Later changes MAY promote specific event kinds to strict-mode blockers.

#### Scenario: Successful build still records degraded fact

- GIVEN a build succeeds after recording a degraded execution fact
- WHEN the pipeline returns the final result
- THEN the result includes the typed hermeticity audit event
- AND the event is available to both human and JSON reporting
