# Specification: Long-lived service readiness

## ADDED Requirements

### Requirement: One readiness vocabulary

r[mantle.service_readiness.readiness_vocabulary] The coordination daemon,
Rust cache daemon, remote serve binding, and source-built fixed-point proof
gates MUST use the declared vocabulary `started`, `ready`, `complete`,
`failed`, plus explicitly declared user-defined states. The coordination
daemon (ADR 0080) MUST report its own readiness. `ready` MUST mean capable
of handling work after a successful real request, not merely started or bound.

Each current assertion MUST be its own `service_readiness` fact. In particular,
`started` and `ready` MUST be concurrently asserted under distinct stable
subjects/IDs (`service/<service_id>/started` and
`service/<service_id>/ready`) while a service is ready; a single latest-state
fact or JSON state union MUST NOT replace the two facts. Fact identity is
`(owner, kind, subject)`; `Fact.state` MUST be bounded, versioned JSON encoding
exactly one assertion, not a plain string. The
`mantle-service-readiness-v1` JSON assertion MUST include `schema`,
`service_id`, `state`, `custom_states`, `dependencies`, `restart_policy`,
`blocked_by` (nullable), and `coordination_state: true`. It MUST fit within
1,536 bytes; IDs MUST use at most 64 ASCII identifier characters, and
declarations MUST contain at most eight dependencies and eight custom states
of at most 48 characters each. Ingress MUST reject an unknown schema or
state, undeclared custom state, malformed/oversize JSON, conflicting
declarations, wrong subject, and a missing or unknown policy. One unique
owner MUST be used per live publisher process session; another simultaneous
session MUST NOT take that owner. Closing the publisher session MUST retract
both current `started` and `ready` facts.

#### Scenario: Coordination daemon and real-request readiness

- GIVEN the coordination daemon starts with no facts in its new generation
- WHEN it starts and can handle one actual subscription request
- THEN it MUST assert `started` and `ready` under two distinct IDs
- AND before that request succeeds it MUST NOT assert `ready`

#### Scenario: Other consumers become ready only after taking work

- GIVEN the Rust cache daemon has loaded its admission policy and remote serve
  has established a binding, but neither has successfully handled a request
- WHEN each component handles its first real admitted request
- THEN each MAY add its distinct `ready` assertion while retaining `started`
- AND a flag, socket bind, remote stdio frame, stdout, or stderr alone MUST
  NOT announce readiness

#### Scenario: Early exit takes precedence

- GIVEN a service has asserted `started` but never `ready`, even if its policy
  is `never`
- WHEN its process exits normally or abnormally before readiness
- THEN it MUST report `failed`, retract any stale `ready`, and MUST NOT report
  `complete` for this early-exit event

#### Scenario: Unknown schema or policy fails admission

- GIVEN a service assertion uses an unknown schema, unknown/undeclared state,
  missing policy, unknown policy, or wrong subject for its asserted state
- WHEN daemon ingress validates it
- THEN it MUST reject the assertion and MUST NOT publish it as current state

#### Scenario: Daemon restart invalidates both assertions

- GIVEN a subscriber sees current `started` and `ready` IDs for a service
- WHEN the coordination daemon restarts or the publisher session closes
- THEN both assertions MUST cease to be current
- AND a subscriber MUST discard prior-generation facts on disconnect/reset
  and MUST wait for new `started` and `ready` assertions

### Requirement: Dependencies are declared and block readiness

r[mantle.service_readiness.declared_dependencies] Each of the four consumers
MUST declare its dependencies. `ready` MUST be admitted only while each
declared dependency is currently `ready` or `complete` and the same service's
`started` assertion is current. `blocked_by` MUST identify a blocking
dependency when blocked; retract `ready` on loss of a required dependency
and re-evaluate the current set before republishing. Source-built fixed-point
proof-stage dependencies MUST derive from existing signed-plan output
references without changing signed-plan bytes or digests. Readiness MUST NOT
activate StageX/protected proof or C cache policy.

#### Scenario: Dependent blocks, recovers, and blocks again

- GIVEN `b` declares `a` as a dependency, `b` is started, and `a` is only
  started
- WHEN `b` evaluates readiness
- THEN `b` MUST report `blocked_by: "a"` and MUST NOT assert `ready`
- WHEN `a` asserts `ready` and `b` successfully handles a real request
- THEN `b` MAY assert `ready` alongside `started` and clear `blocked_by`
- WHEN `a` retracts `ready` without asserting `complete`
- THEN `b` MUST retract `ready` and again name `a` as its blocker

#### Scenario: Signed proof references determine proof blockers

- GIVEN a source-built fixed-point proof stage refers to predecessor outputs
  in the existing signed plan
- WHEN its predecessors are not all currently ready or complete
- THEN the stage MUST be blocked by those declared output references
- AND it MUST NOT rewrite plan digests or bypass protected-proof admission

### Requirement: Restart policy comes from a closed matrix

r[mantle.service_readiness.restart_policy_matrix] Every supervised component
MUST name exactly one required policy `always`, `on-error`, `all`, or `never`
on each runtime assertion. Missing/unknown policy MUST fail admission. The
matrix applies only to the bounded component lifecycle, not to arbitrary
user programs. Subject to the early-exit-failed precedence above:

| Policy | Normal termination | Abnormal termination |
| --- | --- | --- |
| `always` | Restart this process without affecting peers | Restart this process without affecting peers |
| `on-error` | Leave running peers alone; no restart | Restart this process without affecting peers |
| `all` | Leave running peers alone; no restart | Restart the whole named daemon group |
| `never` | No restart; terminal `complete` | No restart; terminal `complete` under the reviewed Synit matrix, except early exit before readiness is `failed` in Mantle |

#### Scenario: Every policy preserves its scope

- GIVEN a supervised service exits after reaching readiness
- WHEN it exits normally and its policy is `always`, `on-error`, `all`, or
  `never`, respectively
- THEN it MUST restart only itself, not restart, not restart, or complete
  without restart, respectively
- WHEN it exits abnormally under those four policies, respectively
- THEN it MUST restart only itself, restart only itself, restart its named
  daemon group, or complete without restart, respectively

### Requirement: Doctor publishes derived readiness state

r[mantle.service_readiness.doctor_derived_state] `mantle doctor` MUST be able
to publish a transient derived coordination-state report, and MUST publish it
only when the daemon is available; an unavailable or nonreading daemon MUST
NOT change doctor results or block a build. Doctor MUST preserve exact
existing human/JSON output bytes and evidence/receipt bytes; it MUST NOT add
fields to doctor stdout. Derived readiness MUST be marked
`coordination_state`, MUST NOT be used for build, store, C cache, protected
proof, or release admission, and MUST NOT be accepted as evidence. One-shot
best-effort publication MUST be capped at 100 ms. Remote stdio frames,
stdout, and stderr MUST NOT transport service readiness.

#### Scenario: Doctor is not evidence

- GIVEN a doctor run with derived readiness and a daemon available
- WHEN it publishes its transient report
- THEN existing human/JSON output and evidence/receipt bytes MUST be exactly
  unchanged, without an additive doctor stdout field
- AND an evidence validator or receipt builder MUST reject coordination state
  as evidence

#### Scenario: Endpoint missing or not reading

- GIVEN a configured coordination endpoint is absent or stops reading
- WHEN a build or doctor attempts best-effort publication
- THEN it MUST finish its one-shot attempt within 100 ms
- AND build outputs and receipts MUST remain identical to those without
  the daemon; no delivery claim is implied
