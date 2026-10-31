# Specification: Native multi-worker goal dispatch

## ADDED Requirements

### Requirement: Worker pools are explicit admitted authority

r[distributed_compilation.pool_selection] Mantle MUST enable pool execution
only through explicit typed operator selection. Every selected endpoint MUST
have a current admitted binding, supported native transport, declared execution
capabilities, separate output trust and caller-authorized credential reference.
Bearers MUST NOT appear in configuration, argv, durable state or reports.
Unknown pools, ambiguous endpoint profiles and unsupported mode/backend
combinations MUST fail before dispatch, upload or store mutation.

#### Scenario: Explicit pool is admitted

- GIVEN a selected pool with two independently bound, credentialed and trusted workers
- WHEN Mantle admits a supported native build
- THEN each worker MUST pass the existing endpoint, capability and trust gates
- AND ticket uses MUST remain scoped to admitted concrete jobs

#### Scenario: Pool label is not authority

- GIVEN an endpoint has a missing/revoked binding, same-name different output key, or no authorized credential
- WHEN pool preflight runs
- THEN Mantle MUST reject that configuration before connecting or uploading
- AND a pool name or local cache permission MUST NOT satisfy the missing authority

### Requirement: One lazy graph distributes ready goals

r[distributed_compilation.goal_dispatch] Mantle MUST dispatch independent
ready derivation goals concurrently to fitting workers through the ordinary
lazy Worker, including dynamically admitted units. Dependency readiness,
starvation policy, attachment/deduplication, job bounds and terminal waiter
notification MUST retain one owner. The worker and store MUST NOT branch on
compiler language or run an independent compiler scheduler.

#### Scenario: Independent goals overlap

- GIVEN a diamond graph with a shared dependency and two eligible workers
- WHEN its independent branches become ready
- THEN real executor lifetimes on distinct workers MUST be able to overlap
- AND the shared dependency MUST execute once and the final goal MUST wait for both verified branches

#### Scenario: Returned plan is not whole-build completion

- GIVEN a verified producer returns a declared plan and source tree
- WHEN the Worker admits those outputs through ordinary dynamic-plan validation
- THEN its ready child units MUST become eligible for the same pool
- AND the selected build MUST NOT succeed merely because its producer finished

### Requirement: Remote actions bind resolved concrete inputs

r[distributed_compilation.concrete_payload] Every dispatched action MUST bind
its registry-resolved derivation/action identity, logical store prefix, complete
declared input/reference closure, tools, ordered arguments, environment,
execution system, sandbox/network policy, required semantic capabilities,
expected named outputs and explicit limits. Placement and attempt metadata
MUST NOT alter semantic action identity. Transfers MUST use admitted,
receiver-requested declared inputs only; workers MUST NOT evaluate raw frontend
source or discover ambient tools, secrets or workspace files.

#### Scenario: A dependent runs on another worker

- GIVEN a goal consumes a verified output built on a different worker
- WHEN its selected worker lacks that output or a transitive reference
- THEN the complete declared missing closure MUST be verified and materialized before execution
- AND the consumer MUST use the same logical input identity as local execution

#### Scenario: Undeclared input is reachable on the client

- GIVEN an action would require a locally reachable but undeclared file or private descriptor
- WHEN remote input planning runs
- THEN Mantle MUST refuse that input rather than upload it implicitly
- AND the action MUST NOT start from an incomplete or fabricated closure

### Requirement: Resource reservations remain current and bounded

r[distributed_compilation.resource_ownership] Pool execution MUST preserve
existing hard placement gates and durable worker/job/attempt/fence reservations.
Global jobs, worker concurrency, quantified resources, live sessions, pending
requests and transfer buffers MUST remain bounded. Reservations MUST commit
before dispatch and reconcile against current durable state on completion.
Unknown quantities MUST stay unknown; concurrency/presence MUST NOT fabricate
CPU, memory, scratch or named-token capacity.

#### Scenario: Concurrent goals share scarce capacity

- GIVEN independent ready goals require more aggregate capacity than a worker owns
- WHEN dispatchers concurrently plan and commit placements
- THEN committed leases MUST never exceed the admitted vector
- AND admitted fitting work MAY use another eligible worker without changing action identity

#### Scenario: Old completion cannot release a new lease

- GIVEN a job was reassigned with a newer attempt/fence
- WHEN its prior worker reports completion or release
- THEN the mutation MUST fail against current ownership
- AND no current lease, other job state or accepted output MUST be overwritten

### Requirement: Output verification precedes completion and publication

r[distributed_compilation.output_admission] A remote result MUST pass current
attempt/fence authorization, exact requested output and prefix binding, full
trusted-key signature verification, required attestations, NAR/content/size and
reference checks before PathInfo/action-result publication or dependent
notification. The coordinator, a successful transport, cache metadata and local
re-signing MUST NOT substitute for that verification. Verified producer plan
and source outputs MUST still pass ordinary dynamic-plan/source admission.

#### Scenario: Trusted output completes a goal

- GIVEN current remote output bytes match the requested identity and full trusted key
- WHEN all existing output-admission checks succeed
- THEN the normal store/goal finalization path MUST admit them once
- AND dependents MUST see the same declared output references as on the local path

#### Scenario: Tampered or misbound output is rejected

- GIVEN a result has changed NAR bytes, an unknown full key, missing attestation, wrong prefix or unexpected output
- WHEN completion is attempted
- THEN Mantle MUST reject the result without publishing PathInfo or an action result for it
- AND no dependent MUST run from those bytes

### Requirement: Terminal failures do not become duplicate success

r[distributed_compilation.terminal_behavior] Compiler rejection MUST remain a
build failure. Trust, content, identity, binding and fence rejection MUST fail
closed without success-producing fallback. Explicit local fallback MAY apply
to admitted pre-dispatch unavailability only. Uncertain dispatched execution
MUST be settled under existing attempt/teardown policy before replacement;
cancellation MUST respect attached waiters and release only owned current
resources. Existing retry/deadline budgets MUST remain bounded.

#### Scenario: Last owner cancels a running job

- GIVEN a remote job has no remaining requesting waiter
- WHEN its last owner cancels
- THEN Mantle MUST stop/drain its owned attempt and observe required teardown before releasing its capacity
- AND a late result MUST NOT complete a successor job or publish stale outputs

#### Scenario: Compiler failure is not a remote outage

- GIVEN a current worker runs a compiler which rejects the source
- WHEN the result reaches the client
- THEN the goal MUST report the compiler failure
- AND Mantle MUST NOT silently retry locally to replace that failure with a successful result

### Requirement: Compatibility and evidence distinguish execution from plans

r[distributed_compilation.compatibility_evidence] Pool selection MUST remain
opt-in. Existing local/single-builder contracts and signed reuse MUST remain
supported; any incompatible wire/report change MUST be versioned with every
consumer migrated. Plan-only selection MUST perform no credential reads,
network, reservations or builds. Bounded reports MUST distinguish local,
remote, cache, failure and unavailable outcomes and bind actual per-goal worker,
attempt, resource, transfer and output-trust observations. They MUST NOT claim
independent-host execution, speedup, compiler correctness or release eligibility
from loopback, scheduling plans or structural lifecycle gates.

#### Scenario: Default local build is unchanged

- GIVEN no pool was selected
- WHEN the ordinary supported local or signed-cache path runs
- THEN its output identity, exit outcome and machine-readable contract MUST remain valid
- AND it MUST NOT be labelled as remote execution by a local service wrapper

#### Scenario: Pool plan does not execute

- GIVEN a valid pool and a request for plan-only output
- WHEN the plan is rendered
- THEN Mantle MUST describe admitted intended routes without reading bearer FDs or opening transports
- AND the report MUST not count an intended placement as executed compilation
