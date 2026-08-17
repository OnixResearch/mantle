## ADDED Requirements

### Requirement: Coordinator runtime schedules workers without becoming output trust

r[remote_builds.coordinator_worker_runtime] Mantle MUST provide a coordinator runtime that records worker registrations, capability facts, concurrency, queue state, live leases, output signing-key identities, and resumable job summaries, then matches concrete build requests by normalized build key, required capabilities, resource policy, upload feasibility, logical store prefix, and client output-trust preflight. The coordinator MUST NOT be treated as an output trust root.

#### Scenario: worker registration enables matching dispatch

GIVEN a worker initiates a coordinator session and registers endpoint identity, systems, feature labels, sandbox modes, network modes, concurrency, transfer capabilities, and output signing-key identities
WHEN a compatible concrete build request enters the coordinator queue
THEN the coordinator MAY assign the job to that worker
AND it MUST NOT assign jobs requiring unadvertised capabilities or mismatched logical store prefix.

#### Scenario: duplicate requests attach to one job

GIVEN two clients submit equivalent concrete requests with the same normalized build key
AND the first job is queued, running, or finished but not fully delivered
WHEN the second request is admitted
THEN Mantle MUST attach the second client to the existing job log/result
AND it MUST NOT start a duplicate build for that normalized key.

#### Scenario: restart adoption preserves phase truth

GIVEN a coordinator or worker restarts while a job is queued, running, transferring, or finished-undelivered
WHEN the worker re-registers and the client resubmits the normalized request
THEN Mantle SHOULD resume or redeliver retained state when the normalized key and trust policy still match
AND it MUST report the exact lost phase instead of silently launching a conflicting duplicate or reporting stale success.
