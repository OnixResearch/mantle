# Rust Package Planning Delta: Bounded Warm Workspaces

## ADDED Requirements

### Requirement: Explicit opt-in provider and capability admission

r[rust_package_planning.warm_workspace.admission] A warm workspace MUST be an explicitly selected host development session with exact provider/backend/protocol/compiler/sysroot/runtime/linker identities, supported fixture/operation rows and a declared authoritative compiler check. Admission MUST require bounded authenticated transport, content-bound update/acknowledgment, compatible output semantics and enforceable isolation/teardown capabilities. Missing required capabilities MUST be typed nonpassing blocked/unsupported outcomes. Default Cargo, ordinary unit execution, wrapper V1 and generic scheduler/store semantics MUST remain unchanged.

#### Scenario: A real qualified provider is explicitly selected

- GIVEN an exact admitted provider and checker closure, finite session policy, declared offline fixture rows and a proven bounded versioned bridge
- WHEN a maintainer explicitly enables warm development execution
- THEN the shell MUST launch the real provider under that policy and bind all requests/results to its exact identities
- AND the disabled/default path MUST retain existing behavior without launching a resident provider or requiring rustd or Headstart.

#### Scenario: Provider or enforcement capability is unavailable

- GIVEN the requested provider lacks exact ABI/toolchain qualification, an authoritative checker, quiescent revision updates, hard memory/process enforcement or owned stop/drain
- WHEN admission evaluates a required warm row
- THEN it MUST retain that row as blocked or unsupported with its precise missing prerequisite
- AND MUST NOT report a successful skip, substitute cold compiler launches as warm reuse, or block independently admitted ordinary baseline delivery.

#### Scenario: Ambient tools or a matching version label cannot substitute

- GIVEN the environment selects a different compiler/sysroot/provider or the backend's ABI/layout compatibility is unqualified despite matching version text
- WHEN session tools and dependencies are resolved
- THEN admission MUST reject the mismatched closure before accepting results
- AND MUST NOT use inherited PATH/toolchain/wrapper settings or successful linking as proof of compatible identities.

#### Scenario: Existing wrapper V1 cannot silently become a resident protocol

- GIVEN the current wrapper contract admits one final output file and no resident revision update, but a request needs retained state or multiple output files
- WHEN the request reaches the daemon
- THEN it MUST require a separately versioned admitted capability/output contract or remain unsupported
- AND MUST NOT reinterpret V1, treat a one-file rename as a multi-file atomic commit, or introduce a second general daemon/CAS to evade existing admission.

### Requirement: Configuration partitions and exact content-bound revisions

r[rust_package_planning.warm_workspace.identity] Resident state MUST be partitioned by semantic workspace configuration, provider/toolchain/backend/protocol closure, dependency graph, target/features/profile/options/environment/policy and output contract. Each ordered request MUST bind session instance, expected base revision and exact requested member/path/content, generated/include/dependency and edit identities using role-labelled BLAKE3. Updates MUST use supported quiescent application and exact acknowledgment, with private input ownership and before/after verification. Mtime, watcher silence, path equality or a body/interface hash MUST NOT replace complete input identity.

#### Scenario: A body edit retains configuration but changes revision

- GIVEN an admitted partition, exact successful base revision and a verified body-edit preimage/postimage with independent expected behavior
- WHEN the supervisor applies the new content at the supported quiescent boundary
- THEN the request and acknowledgment MUST bind that base and new revision while retaining the same admitted configuration partition
- AND result acceptance MUST verify the exact current source/dependency snapshot rather than discard the database merely because source content changed.

#### Scenario: Same-mtime, member and included-file edits still invalidate content

- GIVEN source bytes change without an mtime change, a file is renamed/deleted, or a non-Rust generated/include input changes
- WHEN a new revision is requested even though watcher hints appear unchanged
- THEN the admitted member/content set and absence facts MUST be revalidated and the affected provider update MUST reflect those changes
- AND the shell MUST NOT return unchanged success based on timestamps, old file lists or a surviving artifact path.

#### Scenario: Base or acknowledgment is stale or inconsistent

- GIVEN a request references the wrong base, a response repeats an old revision, or an acknowledgment is unknown, duplicate, reordered or inconsistent with current bytes
- WHEN the application checks the transition
- THEN it MUST reject the transition and invalidate uncertain session state before accepting outputs
- AND no reply from a previous revision or another session instance MUST satisfy the current request.

#### Scenario: Semantic configuration or dependency partition changes

- GIVEN compiler/backend/protocol, manifests/features/profile/options/environment/policy or the dependency graph changes outside the admitted in-partition update capability
- WHEN the next request is considered
- THEN it MUST select a new explicit partition and observe closure/reset of incompatible retained state, or reject the request
- AND MUST NOT reuse old state across that change or count the fresh launch as uninterrupted warm work.

### Requirement: Actual retained query reuse and visible recovery

r[rust_package_planning.warm_workspace.reuse] The implementation MUST retain a real provider's loaded workspace/query state across multiple changed revisions and associate supported structured reuse/invalidation observations with exact revisions and scopes. Provider dependency invalidation MUST include supported structural/upstream/generated/include changes and cached negative/unsupported decisions. Cold launches, whole-result restores and Cargo freshness MUST remain distinct from resident reuse. Reset, crash and recovery MUST preserve state lineage, cause and discarded-state evidence; uncertainty MUST invalidate the session rather than silently retry.

#### Scenario: Changed revisions exercise the same resident workspace

- GIVEN real admitted body/upstream/structural edits with independent expected values and supported provider reuse observations
- WHEN successive revisions compile, check, link and run
- THEN the provider MUST retain loaded state and apply actual affected-query invalidation through the same admitted resident lineage
- AND the report MUST distinguish reused work, newly executed work and unavailable internal metrics with their declared scopes.

#### Scenario: A cached rejection does not poison a repaired revision

- GIVEN a successful revision followed by a real compiler error, a still-broken revision and a repaired revision under a provider that explicitly supports coherent error-state retention
- WHEN the required cumulative campaign continues without a reset
- THEN cached errors/unsupported decisions whose inputs changed MUST be invalidated and each revision MUST meet its independent expected rejection or successful result
- AND a stale rejection, unexplained state reset or missing required recovery step MUST prevent complete warm qualification.

#### Scenario: Cache hits or repeated launches cannot prove warm reuse

- GIVEN an alive broker serves only whole-result hits, repeated cold compiler launches, or Cargo-fresh unchanged builds
- WHEN the feature reports its retained-provider acceptance evidence
- THEN those observations MUST be labelled under their actual mechanisms and MUST NOT satisfy the required changed-source resident-workspace row
- AND low latency, a stable PID, zero functions recompiled or parsed human log text MUST NOT substitute for supported retained-state evidence.

#### Scenario: Provider crash or corrupt state requires observed discontinuity

- GIVEN a resident provider crashes, times out, reports inconsistent protocol state or can no longer prove coherent revision ownership
- WHEN execution stops or an explicitly selected later attempt starts fresh
- THEN the old session MUST be invalidated and torn down, with the failure, discarded state and distinct successor identity retained
- AND the later attempt MUST NOT erase the failure, satisfy the interrupted continuity obligation or be reported as uninterrupted warm recovery.

### Requirement: Checked current outputs and unchanged publication authority

r[rust_package_planning.warm_workspace.publication] Dependencies and provider scratch/fragments/link stages MUST be session-private or immutable pinned inputs. A current result MUST become runnable only after the exact revision's real authoritative compiler check, provider/link success and complete output validation. Output references MUST bind partition/session/revision/attempt. Prior artifacts, raw backend success and fragments MUST NOT authorize current success, ordinary PathInfo, generic goal completion or cache visibility. Existing complete-unit cache eligibility/publication contracts MUST remain unchanged.

#### Scenario: A current checked executable becomes available

- GIVEN provider and checker read the same verified revision and pinned dependency artifacts, and final link/output validation succeeds
- WHEN the application accepts the completed development result
- THEN its runnable output MUST bind the exact partition, session, revision and attempt with complete observed output identities
- AND only unchanged eligible complete-unit cache contracts MAY admit it; private development acceptance MUST NOT create ordinary derivation/PathInfo authority.

#### Scenario: Backend emits code for a revision rejected by its checker

- GIVEN the alternate backend emits a linkable object or binary but the real authoritative check rejects that exact source, is unavailable or checked a different revision
- WHEN current-result eligibility is evaluated
- THEN the result MUST remain non-runnable/nonaccepted with compiler rejection, blocked capability or identity failure classified accurately
- AND provider success or a prior successful checker result MUST NOT authorize executing the current artifact.

#### Scenario: Stale output or mutable dependency artifacts race the current edit

- GIVEN a previous successful executable remains, a concurrent Cargo target mutates a dependency, or current source/dependency verification differs after execution
- WHEN the requested revision fails or output verification runs
- THEN the current attempt MUST reject stale/changed inputs and pending output, retaining any prior complete result only under its own identity
- AND MUST NOT run the old executable as current success or link against a concurrently mutable external target tree.

#### Scenario: Internal fragments have no final-result authority

- GIVEN a provider reuses query/function fragments or produces outputs outside the existing complete-unit cache contract
- WHEN the session reports its work and completed outputs
- THEN fragments MUST remain qualified provider-owned scratch and cache ineligibility MUST be explicit where the final contract is not met
- AND the implementation MUST NOT publish fragment candidates, invent shared fragment trust, signal generic goal completion or widen wrapper/store output semantics.

### Requirement: Finite enforced resident ownership and bounded teardown

r[rust_package_planning.warm_workspace.bounds] Session admission MUST validate positive finite resident/active/revision/task/input/stream/frame/scratch/output/memory/request/lifetime/teardown limits before launch and enforce them at the owning shell boundary. Idle and waiting residents MUST retain their capacity reservations and memory charges. Sampling MUST NOT substitute for hard enforcement. Expiry, cancellation, disconnect, rejection, crash or exhaustion MUST stop admission, invalidate pending results and retain ownership until actual contained-process teardown is observed; incomplete teardown is nonpassing and MUST NOT release unproven capacity.

#### Scenario: Idle resident memory remains charged

- GIVEN the admitted one-resident/one-active small-fixture profile and a warm provider waiting between revisions
- WHEN another request or memory allocation is considered
- THEN idle provider state MUST still consume its resident reservation and enforced session-memory budget
- AND request completion or zero active compilers MUST NOT reset its charge or admit an additional unreserved resident.

#### Scenario: Resource or protocol bounds are exhausted

- GIVEN a real provider/checker tree reaches an enforced memory/task/lifetime limit or an input/frame/stream/scratch/output budget
- WHEN the owning shell observes exhaustion
- THEN it MUST stop accepting revisions, invalidate pending output, terminate/reap the owned tree within the admitted grace and retain a distinct budget/protocol failure
- AND MUST NOT grow unbounded buffers, truncate evidence into a passing result, disable enforcement or retry under an unlimited policy.

#### Scenario: Cancellation or disconnect leaves descendants alive

- GIVEN cancellation/disconnect occurs while provider, checker, link helper or other owned descendants remain active
- WHEN shutdown runs and a teardown deadline expires before empty containment/reap is proved
- THEN the outcome MUST remain incomplete teardown with bounded retained observations
- AND top-level exit, a vanished socket or a dropped future MUST NOT prove cleanup or authorize release of the resident reservation.

#### Scenario: Revision or lifetime budget expires during an idle interval

- GIVEN the session has exhausted its admitted revision count or lifetime while retaining database state
- WHEN a client requests another edit or leaves the session idle
- THEN the shell MUST stop admission and perform the declared bounded stop/drain/teardown policy
- AND MUST NOT keep an immortal uncharged worker or silently reset the budget and resume under the same session identity.

### Requirement: Qualified real comparisons and complete edit-loop evidence

r[rust_package_planning.warm_workspace.qualification] Acceptance MUST exercise real offline provider/compiler fixtures with independently expected per-step behavior and complete frozen case/population accounting. It MUST separately admit same-provider cold/retained comparisons and any different-provider authoritative-reference projection, preserving strict same-binary pipelining rules. Required blocked/unsupported/failed/unrun cells, hidden resets, wrong rejection oracles, mismatches and incomplete teardown MUST remain nonpassing. Performance MUST follow semantic/work qualification and retain cold/bootstrap/first/warm/cache/invalidation/recovery costs, resident resources and observed coverage without changing receipt/cache authority.

#### Scenario: Each required warm step matches a clean checkpoint

- GIVEN a frozen bounded matrix covering supported body/upstream/interface/structural/generic/const/layout/panic/drop and macro/generated/include edits plus cumulative broken/repair/revert/reapply transitions
- WHEN the retained provider and identical independently clean provider execute each applicable exact source state
- THEN statuses, declared diagnostics, expected executable behavior and accepted output identities MUST match under the predeclared fixed policy, with raw identities and lineage retained
- AND ordinary Cargo/native baseline evidence MUST remain independently obtainable without enabling warm sessions or early metadata.

#### Scenario: A different backend has a separately admitted reference projection

- GIVEN an explicitly registered reference comparison binds distinct candidate and authoritative compiler closures and declared semantic/operation projections
- WHEN the same exact fixture revision is checked and built under that qualification
- THEN success/rejection, authoritative structured checks and program behavior MUST meet independent expectations, with raw output digests retained
- AND the report MUST NOT call the comparison same-binary off/on, demand or infer cross-backend machine-code equality, or use descriptive timings as an isolated causal speedup.

#### Scenario: Stable compiled counts cannot hide failed or missing work

- GIVEN a required case/test/denominator member disappears, fallback replaces a required route, a reset hides a failed edit, or a timeout/protocol error is labelled expected compiler rejection
- WHEN population/oracle reconciliation evaluates the report
- THEN qualification MUST be nonpassing despite unchanged compiled totals or agreement between two erroneous sides
- AND all frozen populations, excluded/nonroot reasons, reset history and missing/failed/unrun obligations MUST remain inspectable rather than changing the baseline after observation.

#### Scenario: Cold and idle costs remain visible in a warm report

- GIVEN real qualified executions include bootstrap, first edit, later warm/no-op/cache/invalidation/recovery steps plus failures and incomplete resource/phase samples
- WHEN wait-diagnostics summarizes the declared comparable cohorts
- THEN it MUST retain those distinct costs and work populations, raw/first-edit samples, per-scenario counts/distributions and exact resident/peak/idle sampling/lifetime coverage
- AND unavailable phases/counts MUST remain unavailable, zero recompilation MUST NOT imply zero latency, and upstream rustd figures or omitted setup costs MUST NOT become claimed Mantle savings.
