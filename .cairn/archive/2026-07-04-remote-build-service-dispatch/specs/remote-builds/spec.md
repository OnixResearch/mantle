## ADDED Requirements

### Requirement: Remote build dispatch integrates with the lazy scheduler

r[remote_builds.scheduler_build_service_dispatch] Mantle MUST expose remote build execution through a scheduler-compatible dispatch boundary so ready derivation or action goals can be realized remotely without bypassing lazy goal deduplication, dependency interleaving, route policy, or output admission. The remote dispatcher MUST accept only concrete evaluated build inputs and MUST return outputs through signed PathInfo, artifact-attestation, and store import validation before a goal is marked done.

#### Scenario: scheduler dispatches a ready goal remotely

GIVEN a ready goal has a selected remote-builder route
AND the remote build service receives concrete build inputs, an upload manifest, and trusted output-admission facts
WHEN the scheduler dispatches the goal
THEN Mantle MAY run the remote build through the remote service
AND the goal MUST be marked done only after verified remote outputs are imported through the ordinary store path.

#### Scenario: identical ready goals dedupe across remote dispatch

GIVEN two requested roots normalize to the same concrete build key
AND the first request has already queued, started, or finished a remote dispatch that still owns the result
WHEN the scheduler receives the second request
THEN Mantle MUST attach the second request to the existing goal/result
AND it MUST NOT start a duplicate remote build for the same normalized key.

#### Scenario: invalid remote request cannot bypass scheduler policy

GIVEN a remote route would require raw Nickel evaluation, frontend module interpretation, mismatched store prefix, invalid fallback policy, or untrusted output admission
WHEN the scheduler evaluates dispatch
THEN Mantle MUST reject the remote route before marking the goal complete
AND it MUST preserve a phase-classified failure instead of fabricating a local or remote success.
