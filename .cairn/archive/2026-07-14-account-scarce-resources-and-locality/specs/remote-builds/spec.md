# Remote Builds Specification

## Purpose

Define durable fenced reservation of quantified remote-worker resources.

## ADDED Requirements

### Requirement: Remote worker resources use fenced leases
r[remote_builds.fenced_worker_resource_leases]

Mantle MUST reserve quantified worker capacities and named scarce tokens through durable leases bound to the normalized job, current attempt, fence generation, worker identity, requirement digest, and reservation digest before remote assignment. Lease authorization MUST control scheduling capacity only and MUST NOT establish tool identity, output trust, attestation validity, or license compliance.

#### Scenario: Current assignment owns one reservation

- GIVEN a compatible worker has sufficient unreserved capacity
- WHEN the coordinator commits an assignment
- THEN it MUST durably commit exactly one current job/attempt/fence-bound reservation before instructing the worker to execute
- AND concurrent assignments MUST observe the committed remaining capacity.

#### Scenario: Stale attempt cannot mutate capacity

- GIVEN reassignment advanced the current attempt or fence
- WHEN the superseded worker tries to renew, release, resize, or complete its old resource lease
- THEN Mantle MUST reject the mutation with a stale-fence diagnostic
- AND current reservation and job state MUST remain unchanged.

#### Scenario: Restart recovers conservatively

- GIVEN the coordinator or worker restarts with durable active resource leases
- WHEN registrations and job state are reconciled
- THEN Mantle MUST preserve or release each reservation through an explicit current-state transition
- AND it MUST NOT silently double-allocate capacity or report a leaked reservation as successful execution.

#### Scenario: Named token does not confer output trust

- GIVEN a worker obtains a named scarce token such as a licensed-tool seat
- WHEN the build completes
- THEN the token MUST count only as resource authorization evidence
- AND returned outputs MUST still pass ordinary action identity, PathInfo, signature, attestation, and producer-policy admission.
