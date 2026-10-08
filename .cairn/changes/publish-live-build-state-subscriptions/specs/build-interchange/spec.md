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

The live daemon owner id MUST identify the individual emitting invocation.
The content-addressed evaluation-stream run id MUST remain unchanged and MUST
NOT be reused as live owner identity for concurrent identical inputs.

#### Scenario: Build without a coordination daemon

- GIVEN a build with live-state emission enabled and no daemon reachable
- WHEN the build runs to completion
- THEN it MUST produce the same outputs and receipts as a build with emission
  disabled
- AND it MUST report degraded observation

#### Scenario: Concurrent identical-input builds remain independent

- GIVEN two simultaneous builds with identical source and selected roots
- WHEN both opt into the same live daemon
- THEN each MUST publish under a different owner id without duplicate-owner
  degradation
- AND stopping either build MUST NOT retract the other build's facts
- AND their NDJSON/report identities, outputs, and receipts MUST remain unchanged

#### Scenario: Emission does not block dispatch

- GIVEN a slow or stalled coordination endpoint
- WHEN the scheduler dispatches goals
- THEN emission MUST NOT delay dispatch beyond the declared emission bound
