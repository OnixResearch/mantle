# Remote Builds Specification

## Purpose

Define authority-safe leases for mutable tool workspaces on remote workers.

## Requirements

### Requirement: Remote stateful workspaces use bounded fenced leases [r[remote_builds.stateful_workspace_leases]]

Mantle MUST bind each mutable remote workspace lease to a worker identity, authority class, action-compatibility digest, toolchain refs, stable guest mount path, current job, attempt, fence generation, quota policy, and retention class. A stale attempt, different worker, different authority, incompatible action/toolchain, concurrent owner, or unknown cleanup state MUST NOT read or mutate the workspace.

#### Scenario: Current compatible attempt reuses workspace

- GIVEN a worker holds a compatible bounded workspace and the current job/attempt/fence acquires its exclusive lease
- WHEN the remote sandbox starts in mutable-session mode
- THEN Mantle MAY mount the workspace at the declared stable guest path
- AND status/build evidence MUST identify warm-state use and its narrower claim class without exposing host paths or workspace contents.

#### Scenario: Stale or foreign owner is rejected

- GIVEN a workspace lease belongs to a superseded attempt, different worker, different authority class, incompatible action/toolchain, or another active owner
- WHEN a remote request asks to mount, renew, snapshot, scrub, or delete it
- THEN Mantle MUST reject the mutation before sandbox start or filesystem change
- AND current lease and workspace state MUST remain unchanged.

#### Scenario: Failed cleanup quarantines state

- GIVEN a remote build ends and required scrub, bounded scan, snapshot, or cleanup cannot complete
- WHEN Mantle transitions the workspace lease
- THEN it MUST quarantine the workspace and block subsequent reuse
- AND it MUST report cleanup failure separately from execution/output truth while withholding any claim that requires successful cleanup.

#### Scenario: Retention preserves active leases

- GIVEN workspace retention or worker garbage collection runs
- WHEN active, idle, expired, and quarantined workspaces are classified under policy
- THEN Mantle MUST preserve current active leases and apply only the accepted bounded eviction plan
- AND stale metadata or storage pressure MUST NOT authorize deletion of a current workspace.
