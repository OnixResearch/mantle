# Specification: Independent-host compilation qualification

## ADDED Requirements

### Requirement: Qualification binds explicit admitted inputs and hosts

r[distributed_compilation.qualification_manifest] Mantle MUST provide a
repository-owned explicit qualification workflow driven by a bounded,
Nickel-authored exported manifest. It MUST bind workload/tool/policy/logical
prefix/backend identities, expected outcomes, required cells, resource limits,
operator-authorized endpoints/bindings and credential references without
bearers. It MUST separate loopback and independent-host modes and preflight
missing/invalid prerequisites before remote effects. The runner MUST NOT
provision hosts/secrets, infer upload authority or substitute an easier mode.

#### Scenario: Operator supplies a complete independent-host manifest

- GIVEN exact supported workloads and two explicitly authorized independent worker hosts
- WHEN qualification preflight succeeds
- THEN the runner MUST bind the selected identities, capabilities and host/session provenance
- AND execution MUST remain within supplied credential, source and resource authority

#### Scenario: Independent hosts are unavailable

- GIVEN a required host, binding, FD, tool or capability is missing
- WHEN independent-host qualification is requested
- THEN it MUST return an explicit unavailable nonpassing result naming the missing prerequisite
- AND local stdio or skipped cells MUST NOT satisfy independent-host acceptance

### Requirement: Qualification observes real distributed compiler execution

r[distributed_compilation.multihost_execution] Required acceptance MUST run
real Rust and C/C++ compilers through production native adapters on two
operator-supplied independent worker hosts with separate stores/keys. It MUST
include branching graphs, real proc-macro/build-script/generated-source use,
C/C++ archive/link/install consumers and a Rust consumer of a declared native
provider. Actual current compiler lifetimes on distinct workers MUST overlap,
and final programs MUST execute with independently expected results. Wrapper
starts, placement/cache counts and endpoint labels MUST NOT prove compilation
or independent hosts. Cross-host overlap MUST use qualified timing/liveness
observations, not assume synchronized wall clocks.

#### Scenario: One graph compiles on two independent hosts

- GIVEN a supported branching graph and two admitted independent worker hosts
- WHEN the required two-worker cold run executes
- THEN retained current compiler execution/liveness observations MUST establish overlap on both hosts
- AND final verified outputs MUST be used by a correctly running application

#### Scenario: A native library feeds Rust

- GIVEN a declared C/C++ native-provider graph and a Rust script/link consumer using its output role
- WHEN the mixed-language pool build completes
- THEN the Rust consumer MUST use the current verified provider and complete runtime closure
- AND the executed final program MUST reflect the independent expected native-library result

#### Scenario: Two local labels are presented as independent hosts

- GIVEN two endpoint labels/processes or isolated stores without independent-host provenance
- WHEN the runner classifies the observation
- THEN it MUST remain local integration or unavailable independent-host evidence
- AND it MUST NOT count as a passed multi-host row

### Requirement: Negative controls prevent false distributed success

r[distributed_compilation.qualification_failures] Qualification MUST exercise
real compiler/script failures after prior success, stale artifacts/fences,
wrong full keys, corrupt bytes, misbound outputs, resource contention/shrink,
worker/transport loss, cancellation/timeout, transfer resume/corruption and
private-input/binding rejection through production paths. It MUST inspect
expected semantic outcomes, store publication, current attempt/lease state,
attached-waiter behavior and owned teardown. Infrastructure failure MUST stay
distinct from expected rejection; fallback MUST NOT erase required failures.

#### Scenario: A signed old result is replayed after reassignment

- GIVEN a new current attempt and a validly signed output for an older fence
- WHEN the old result is delivered
- THEN no current completion, lease release or successful publication MUST be accepted from it
- AND the report MUST preserve the stale-scope rejection and current attempt state

#### Scenario: Loss occurs while compilation is active

- GIVEN real worker compilation with current reservations and attached waiters
- WHEN the worker disconnects, times out or its last owner cancels
- THEN settlement/replacement MUST respect current ownership and required observed teardown
- AND no duplicate accepted result, leaked owned process or invented free capacity MUST occur

#### Scenario: A private file is reachable outside admitted inputs

- GIVEN a positive outside-sandbox read of a private marker and no declared input authority for it
- WHEN the remote compiler action executes or input planning rejects the request
- THEN the marker MUST NOT be uploaded or readable by that action
- AND the retained control MUST distinguish a real denial from an absent fixture

### Requirement: Qualification is repeatable and claims are bounded

r[distributed_compilation.repeatable_evidence] The runner MUST repeat cold
qualification from fresh owned client/worker state and controlled verified-warm
and input-change runs. Bounded deterministic reports MUST bind exact commands,
exits, raw evidence and selected artifacts with BLAKE3 and retain every required
cell, actual compiler count, transfer/reuse cost, failed/timeout/unavailable
attempt, final program result and comparison verdict. Same-plan identity and
cross-lane semantic comparisons MUST stay distinct. Missing cells, changed
subjects/evidence, false attribution and hidden failures MUST invalidate the
report. Support and performance statements MUST be restricted to recorded
workloads/hosts/tools/policies; they MUST NOT claim general speedup, arbitrary
worker honesty, compiler correctness, bootstrap, global reproducibility or
release eligibility. Lifecycle gates alone MUST NOT qualify execution.

#### Scenario: Fresh-state repeat agrees

- GIVEN the same admitted manifest and new independent store/state roots
- WHEN the complete cold matrix is repeated
- THEN program outcomes and required same-plan artifact/comparison identities MUST agree
- AND measured execution/transfer observations and all attempts MUST remain inspectable

#### Scenario: A required failed run is omitted

- GIVEN a report omits a required cell or discards a failed attempt in favor of a retry
- WHEN the qualification verifier checks manifest/evidence linkage
- THEN it MUST reject the report as incomplete
- AND a successful subset, cache hit or passing structural gate MUST NOT repair that omission

#### Scenario: A small distributed build is slower

- GIVEN comparable retained local and multi-worker measurements for a declared workload
- WHEN transfer overhead exceeds saved compiler time
- THEN the report MUST retain that cost and the unchanged workload/denominator
- AND it MUST NOT manufacture a speedup claim or weaken the acceptance matrix
