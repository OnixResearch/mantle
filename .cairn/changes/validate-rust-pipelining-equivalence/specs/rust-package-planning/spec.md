# Rust Package Planning Delta: Differential Execution

## ADDED Requirements

### Requirement: Independently runnable admitted differential matrix

r[rust_package_planning.differential_execution.admission] The differential harness MUST deliver real ordinary supported Cargo baseline replay and incremental-versus-clean comparisons without requiring pipeline implementation, and MUST admit serial/native or future experimental cells only through an explicit bounded compatibility matrix with typed nonpassing unsupported/blocked outcomes.

#### Scenario: Ordinary baseline runs without an experimental compiler

- GIVEN declared offline small fixtures, supported pinned ordinary Cargo/rustc binaries, an executable target and admitted process isolation
- WHEN the maintainer runs the required baseline build and separate check cells
- THEN the harness MUST run real compilation in independent sides and report baseline-replay and incremental-versus-clean evidence
- AND it MUST NOT require early metadata, a Headstart checkout or any pipeline feature to establish the baseline.

#### Scenario: Missing experimental capability cannot become a passing skip

- GIVEN a requested experimental mode lacks a registered capability or a required selected row lacks its compiler, target, linker or source prerequisite
- WHEN matrix admission evaluates the row
- THEN the harness MUST retain the row as typed unsupported or blocked with the exact reason
- AND a required row MUST prevent complete-green acceptance rather than count as skipped success
- AND an explicitly unsupported optional future mode MUST NOT prevent independent delivery of the fully exercised ordinary baseline.

#### Scenario: Feature support is bounded per cell

- GIVEN a matrix with ordinary dependency-chain, representative proc-macro, build-script and selected LTO rows
- WHEN one target/profile/operation supports a feature and another does not
- THEN the report MUST bind support to each exact adapter/mode/operation/host/target/profile/features row
- AND it MUST NOT infer check-mode LTO execution, global proc-macro/build-script support, or full Mantle per-unit self-hosting from another row's success.

### Requirement: Exact paired identities and isolated attempt trees

r[rust_package_planning.differential_execution.identity] Each comparison MUST bind complete source/compiler/Cargo/sysroot/tool/options/policy identities and exact private input/artifact trees, MUST preserve semantic order, and MUST compare future off/on modes with the same pinned binaries except for explicitly declared treatment inputs.

Same-binary pipelining registration MUST NOT authorize a different-provider reference comparison. Any future such comparison MUST be separately admitted as qualification with explicit provider/compiler/sysroot identities and a predeclared operation-specific semantic projection; it MUST NOT claim an off/on treatment or raw artifact-byte equivalence. Ordinary Cargo baseline delivery MUST remain independent of that qualification, rustd and warm-session implementation.

#### Scenario: Equal source states retain independent mutable roots

- GIVEN a baseline replay or registered experimental pair with a declared fixture and edit sequence
- WHEN the shell materializes the two sides
- THEN source member contents and declared dependencies MUST match their expected digests
- AND source, target, incremental, HOME, TMPDIR, Cargo home, generated output, store/state, cache, lock and protocol roots MUST be attempt-owned and disjoint between sides
- AND intentional incremental carryover MUST occur only within the declared side and step sequence.

#### Scenario: Different toolchains or contaminated roots are rejected

- GIVEN the on side resolves a different rustc/Cargo/sysroot or a private root aliases another side, prior incomplete attempt, or undeclared ambient target/cache
- WHEN the pair is admitted
- THEN it MUST be rejected as noncomparable before an equivalence claim
- AND a retry MUST use a new exact attempt root rather than consume an incomplete attempt's state.

#### Scenario: Replay identity retains ordered semantic inputs

- GIVEN ordered argv, admitted environment, source/lock/config/dependency closure, target/profile/features, compiler/Cargo/sysroot/linker closure, cache policy, edit manifest and comparison policy
- WHEN logical cell and step identities are constructed
- THEN the report MUST bind those facts with versioned role-labelled identities and retain exact attempt locations separately
- AND it MUST preserve ordered arguments, edits, per-unit events and duplicates rather than treating them as sorted sets.

### Requirement: Verified real incremental edit sequences

r[rust_package_planning.differential_execution.incremental] The harness MUST use real compilation and verified exact source edits for cold, warm, unchanged, body-only, interface, opaque, generic, const, async, break/fix/edit/revert and final incremental-versus-clean sequences, with independent expected behavior and accepted output-identity checks.

#### Scenario: Edits and unchanged rebuilds have observable meaning

- GIVEN an admitted dependency-to-consumer fixture with exact edit preimages/postimages and independent expected printed values or diagnostics
- WHEN cold, warm unchanged, forced unchanged-source incremental recompile, body-only, interface, opaque hidden-type, generic, const and async steps execute
- THEN each changed step MUST prove its expected match count and before/after source identities and observe the intended rebuild and expected behavior/error
- AND unchanged steps MUST prove equal source identities and distinguish Cargo freshness from actual compiler execution with retained incremental state
- AND inability to force the required unchanged-source compiler invocation MUST be blocked rather than passed as a freshness check.

#### Scenario: Missed edits and stale outputs cannot agree falsely

- GIVEN an edit no longer matches its preimage, has an unexpected match count, or should change an independent runtime sentinel
- WHEN both sides would otherwise report equal output from old artifacts
- THEN the harness MUST fail the edit/observation step
- AND equality between the sides alone MUST NOT satisfy the expected behavior.

#### Scenario: Break, fix, edit and revert end in independently clean behavior

- GIVEN a successful base followed by dependency body failure, fix, later consumer edit and revert
- WHEN both sides finish the declared successful final source state and compile byte-identical final sources in independent clean roots
- THEN incremental and clean statuses, ordered semantic diagnostics, executable behavior for build rows and accepted output identities MUST agree under the predeclared versioned policy
- AND raw artifact digests and any explicitly admitted nonsemantic identity exclusions MUST be retained
- AND an ICE or SVH disagreement MUST fail even if reproduced on both sides; disabling incremental or metadata hashing MUST NOT turn the original row green.

### Requirement: Cumulative stateful campaigns with independent step oracles

r[rust_package_planning.differential_execution.stateful_campaign] The harness MUST carry declared side-private incremental state through bounded cumulative successful, broken, still-broken, repaired, later structural/upstream/include/macro/generated-input edit and revert/reapply campaigns, MUST compare every applicable step with an independently materialized clean source state and predeclared expected behavior/diagnostics, and MUST expose recovery without erasing contamination or failure.

#### Scenario: Cumulative edits retain state and match clean expectations

- GIVEN a frozen bounded campaign family covering successful -> broken -> still broken with another-file edit -> repaired -> later edits across required applicable structural, upstream-dependency, include-file, macro and generated/build-script-input classes -> revert/reapply
- WHEN the ordinary build and separate check sides execute their steps using retained side-private incremental state and independently materialized fresh clean references
- THEN every applicable step MUST verify exact source/config/input identities, including additions/deletions and non-Rust included bytes, against the manifest rather than copying the campaign tree as the clean oracle
- AND status and ordered semantic diagnostics MUST agree with both the predeclared independent expectation and the clean reference; successful build steps MUST additionally observe current expected executable behavior
- AND source revert/reapply MUST NOT reset retained incremental state, while final clean accepted-output identity checks and build/check separation MUST remain required.

#### Scenario: Wrong negative oracle fails even when both sides agree

- GIVEN an alleged broken step whose predeclared oracle requires a specific real compiler rejection, but independently materialized clean source is accepted by the real compiler or rejected for different semantic diagnostics
- WHEN campaign and clean observations are compared, even if they agree with one another
- THEN the harness MUST retain the expected rejection, actual exit and raw/structured diagnostics and report a nonpassing oracle mismatch
- AND it MUST NOT reinterpret the source as expected success or change the rejection oracle after observation to make the step pass.

#### Scenario: Infrastructure failure is not expected compiler rejection

- GIVEN a declared broken step instead encounters crash, ICE, timeout, link failure, protocol failure, incomplete/malformed capture or cleanup failure
- WHEN either or both sides report an error
- THEN the harness MUST classify and retain the distinct nonpassing failure with its actual process and observation evidence
- AND a generic error reply, matching nonzero statuses or identical failures MUST NOT satisfy the expected ordinary compiler-rejection and semantic-diagnostic oracle.

#### Scenario: Recovery records discontinuity rather than cleansing failure

- GIVEN a failed or contaminated campaign cannot continue safely without a restart, retained-state reset or fallback
- WHEN an explicitly admitted recovery proceeds, or the campaign stops within bounds
- THEN recovery MUST record its cause, predecessor/successor attempt or session identities, failed observations, exact discarded/preserved state and teardown as an observed transition
- AND the original failure and unmet uninterrupted-continuity obligation MUST remain nonpassing; remaining required steps MUST be retained as unrun if not executed
- AND an automatic restart, target deletion, cache reset or new session MUST NOT silently replace the failed transition or turn later success into proof of continuous correctness.

### Requirement: Semantic diagnostics and operation-specific behavior

r[rust_package_planning.differential_execution.semantics] Equivalence MUST compare structured diagnostic semantics, status, multiplicity and per-unit ordering with only narrowly declared timing/progress and unrelated cross-crate interleaving exclusions; build/check and fault-oracle evidence MUST remain distinct.

#### Scenario: Equivalent diagnostics retain unit ordering and duplicates

- GIVEN raw compiler/Cargo records mapped to exact unit/attempt identities, including a repeated warning and ordered notes/suggestions
- WHEN comparison applies its versioned root-role and timing/progress exclusions
- THEN diagnostic level/code/message/spans/children/suggestions/applicability, duplicate count, ordered per-unit events and final exit/signal status MUST be compared
- AND unrelated cross-crate interleaving MAY differ only without erasing dependency-before-consumer terminal-output constraints
- AND the original raw streams MUST remain available.

#### Scenario: Sorting or broad filtering cannot hide a regression

- GIVEN a side duplicates/drops/reorders a per-unit diagnostic, changes a replacement suggestion, produces an unknown semantic field, or emits malformed/incomplete structured output
- WHEN a globally sorted or broadly filtered transcript would look equal
- THEN the harness MUST report a semantic mismatch or typed nonpassing observation failure
- AND it MUST NOT sort, deduplicate, ignore all stderr/rendered diagnostics, or exclude panic/ICE text to manufacture equality.

#### Scenario: Check success does not establish executable behavior

- GIVEN separate required build and check rows for a fixture
- WHEN the check row succeeds and the build row fails or has no current successful executable
- THEN only check compilation evidence MUST be reported for that row
- AND no stale executable MUST be run as current build success
- AND crash/cancel rows MUST use their explicit fault/status oracle rather than comparison against normal successful-run output.

### Requirement: Real failure boundaries and bounded process teardown

r[rust_package_planning.differential_execution.failure] The harness MUST exercise real compilation failure, producer crash and cancellation, MUST prove process-tree teardown and absence of accepted provisional publication, and MUST require observed early-publication boundaries before claiming future after-metadata or swap coverage.

#### Scenario: Ordinary failure and injected crash terminate private attempts

- GIVEN real dependency/application error sources and separately declared crash/cancel attempts running real producer processes in private containment
- WHEN the compiler fails, the controller crashes the producer, or cancellation is requested
- THEN exit/signal and descendant statuses MUST be retained and all consumer, waiter, build-script and helper processes MUST terminate and be reaped within the admitted teardown deadline
- AND no provisional artifact, successful result record, signed envelope or visibility marker for the failed attempt MUST become accepted
- AND previously complete unrelated results MUST be distinguished by producer/attempt identity rather than treated as provisional failures.

#### Scenario: Stale prior final artifact cannot satisfy current failure

- GIVEN a genuine successful earlier step left a final executable/library in adversarial prior state and the current step's producer fails or is cancelled
- WHEN output/publication checks evaluate the current attempt
- THEN the prior artifact MUST NOT prove current success, be executed as a current result, or authorize current cache publication
- AND top-level child exit alone MUST NOT prove teardown while any descendant or waiter remains alive.

#### Scenario: Future metadata mode proves the boundary before claiming coverage

- GIVEN an explicitly registered experimental compiler mode supports observable early publication and matching full-metadata swap
- WHEN a real producer body error, crash or cancellation occurs after its verified exact-attempt publication event, or a consumer resumes after matching full metadata arrives
- THEN the report MUST bind the boundary and consumer-wait/swap observations to those attempts and verify the corresponding failure cleanup or successful final behavior
- AND a sleep, synthetic compiler event, finished-before-injection producer or unavailable protocol MUST NOT count as after-metadata/swap proof; that row MUST remain nonpassing or unsupported/blocked as applicable.

### Requirement: Bounded shell and uncontaminated environments

r[rust_package_planning.differential_execution.bounds] The runner MUST enforce predeclared finite resource and deadline bounds, use explicit child environments without in-process global mutation, and classify incomplete capture or cleanup as nonpassing outcomes.

#### Scenario: Admitted bounded execution is repeatable

- GIVEN concrete validated maxima for cells, steps, units/processes, source/artifact/scratch bytes, streams, diagnostics, edits, build/program/probe/total deadlines and teardown grace
- WHEN an admitted matrix runs with one concurrent cell and explicit allowlisted child environments
- THEN it MUST remain inside those bounds, use declared offline tool/source inputs, and record the bounds/policy identities in its report
- AND a repeat in fresh roots MUST preserve semantic verdicts without depending on ambient RUSTC, PATH, Cargo cache or process-global environment changes.

#### Scenario: Overflow and cleanup failure cannot masquerade as equality

- GIVEN a stream/scratch bound, deadline, permission or storage failure, or a descendant that cannot be proven reaped
- WHEN the runner cannot complete capture or teardown
- THEN it MUST terminate the owned process tree, retain bounded failure evidence and report a typed nonpassing harness/environment outcome
- AND it MUST NOT compare truncated prefixes as complete diagnostics, retry without a bound, mutate the parent environment, or silently discard the failed cell.

### Requirement: Retained repeatable evidence and honest completion

r[rust_package_planning.differential_execution.evidence] Every reported verdict MUST retain raw observations and full replay identities, distinguish required coverage from unavailable/unrun cells, and claim completion only for actually exercised admitted behavior.

#### Scenario: Another maintainer repeats an admitted report

- GIVEN a report containing the exact matrix, required flags, all source/edit/compiler/sysroot/options/policy identities, raw stdout/stderr/JSON, statuses, artifacts, semantic comparisons and cleanup/publication observations
- WHEN another maintainer uses those declared inputs in fresh private roots
- THEN the report MUST provide enough bounded replay information to reconstruct the requested steps and expected independent sentinels
- AND source edits, status mismatches, ICEs and teardown failures MUST remain inspectable rather than replaced by a single green summary.

#### Scenario: Unrun experiments do not inherit baseline success

- GIVEN ordinary baseline cells succeeded but an upstream experimental mode was unregistered, unavailable, blocked or not executed
- WHEN the package's evidence is summarized
- THEN it MUST label baseline completion separately and explicitly retain the experimental row's actual status
- AND it MUST NOT claim Headstart support, complete experimental equivalence, full-workspace self-hosting, release reproducibility or measured Mantle speedup.

### Requirement: Frozen population and coverage reconciliation

r[rust_package_planning.differential_execution.population] Every report MUST reconcile a frozen bounded manifest and retained coverage baseline using stable role-labelled population identities, explicit denominator member sets and selected/eligible/attempted/executed/restored-or-reused/unsupported/fallback/excluded/failed/unrun dimensions; coverage loss MUST remain nonpassing independently of absolute compiled count or semantic equality among surviving cases.

#### Scenario: Complete population retains overlapping dimensions and nonroots

- GIVEN predeclared bounded candidate cases, selected required/optional cell-side-step obligations, expected test IDs and planned unit identities with exclusion/nonroot reasons
- WHEN a report records compilation, complete-result restoration, incremental reuse, fallback, unsupported, failed and unrun observations
- THEN it MUST retain every population member and define selected membership, policy eligibility, begun attempts, observed execution, identified restore/reuse, route, exclusion and verdict separately
- AND it MUST permit overlap across dimensions, such as an attempted executed step that reused units and failed, while reconciling unique member keys and only declaring disjoint arithmetic within a named role/dimension
- AND it MUST identify completion, test and compilation/reuse denominators by exact frozen member sets, keep retries/event counts separate from unique coverage, and retain excluded/nonroot populations without implying they were compiled.

#### Scenario: Stable compiled totals cannot conceal disappeared population

- GIVEN a retained baseline and frozen manifest with a required case, expected test or eligible denominator unit that disappears, is relabelled excluded/nonroot, or is replaced by another member
- WHEN a report preserves or increases the absolute compiled count and surviving behavior still matches
- THEN population reconciliation MUST identify the missing or changed member and report nonpassing coverage with the old/new denominator identities and membership diff
- AND it MUST NOT pass by reducing the denominator, forgetting a test, dropping a case or counting a replacement as unchanged coverage.

#### Scenario: Required skips and fallback remain visible coverage deficits

- GIVEN a required selected observation becomes blocked, unsupported, skipped, failed or unrun, or runs only through an alternative fallback route
- WHEN coverage is summarized, including a run with increased required skips but unchanged compiled totals
- THEN every obligation MUST retain its required flag, reason, actual route and attempted/executed/reused status; requested-route deficits and increased required skips MUST remain nonpassing
- AND optional unavailable future modes MUST remain visible without entering the ordinary baseline's required denominator or blocking its independent delivery
- AND a fallback or restored result MUST NOT masquerade as execution of an unobserved required compiler/program/test route.

#### Scenario: Reports cannot silently replace their coverage baseline

- GIVEN a report would improve by rewriting its retained baseline, required flags or denominator after observing coverage loss
- WHEN the report is finalized or a population revision is proposed
- THEN it MUST compare against the frozen manifest/baseline identities and retain the original regression rather than updating the baseline in place
- AND any intentional revision MUST require separately reviewed versioned manifest/baseline admission with explicit member changes and reasons, retain both identities, and MUST NOT retroactively repair the failed prior report.

### Requirement: Separate time-to-error observations

r[rust_package_planning.differential_execution.error_timing] Time-to-error MUST be reported separately from semantic equivalence, with monotonic timestamp provenance and explicit unavailability of first-error or wait detail when not actually observable.

#### Scenario: Observed timing explains but does not decide equivalence

- GIVEN a pair with equivalent diagnostics and statuses and measured invocation-to-terminal-error/teardown times
- WHEN future attempt-bound wait diagnostics or a justified live actionable-error observation are also available
- THEN the timing report MUST identify first-actionable-error, terminal-error, teardown and observed wait intervals with their exact timestamp origins
- AND differing elapsed time MUST NOT alter the semantic equivalence verdict.

#### Scenario: Completion capture cannot invent live error timing

- GIVEN only the existing completion-oriented capture and no attempt-bound live error/wait observations
- WHEN a failure comparison is reported
- THEN first-actionable-error and wait timing MUST be explicitly unavailable while observed terminal timing MAY be reported
- AND the harness MUST NOT infer zero wait, backdate a buffered diagnostic as a live error, or report an unmeasured Mantle speedup.
