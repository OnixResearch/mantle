# Specification: Live build-state emission

## ADDED Requirements

### Requirement: The building plane emits bounded live facts

r[mantle.build_interchange.live_state_emission] The building plane MUST emit
bounded, versioned facts and events for goal state, worker presence,
reservations, and terminal outcomes. Fact identity MUST come from the building
plane.

Emission MUST NOT block, MUST NOT change build behavior, and MUST NOT be a
build precondition. A missing or failed coordination endpoint MUST produce a
degraded observation event and MUST NOT fail the build.

#### Scenario: Build without a coordination daemon

- GIVEN a build with live-state emission enabled and no daemon reachable
- WHEN the build runs to completion
- THEN it MUST produce the same outputs and receipts as a build with emission
  disabled
- AND it MUST report degraded observation

#### Scenario: Emission does not block dispatch

- GIVEN a slow or stalled coordination endpoint
- WHEN the scheduler dispatches goals
- THEN emission MUST NOT delay dispatch beyond the declared emission bound
