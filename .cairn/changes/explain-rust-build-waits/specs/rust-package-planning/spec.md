# Rust Package Planning — Wait Diagnostics Delta

## ADDED Requirements

### Requirement: Exact invocation, unit, and compiler-attempt identity

r[rust_package_planning.wait_diagnostics.identity] Opt-in Rust wait diagnostics MUST bind observations to an invocation-local unit and actual process attempt using package/source identity, target name/kind, host/target roles and triples, feature set, profile, mode, resolved compiler/input identities, and explicit dependency-unit edges. Default Cargo execution MUST obtain that binding through explicit admitted wrapper/event integration, not crate-name matching, output basenames, PID identity, timestamps, or human-log parsing. Ambiguous binding MUST remain unmapped and unavailable for unit attribution.

#### Scenario: Same crate name identifies distinct real attempts

r[rust_package_planning.wait_diagnostics.identity.distinct]

- GIVEN a local session compiles a host proc macro and target library with the same crate name, and an admitted fallback repeats one compiler invocation
- WHEN the opt-in diagnostic records their actual launches and completions
- THEN each record MUST retain its distinct unit/configuration and role, invocation ID, attempt number, and resolved-process identity
- AND the fallback MUST link its actual prior attempt rather than overwrite the failed attempt.

#### Scenario: Cargo wrapper cannot prove a package association

r[rust_package_planning.wait_diagnostics.identity.unmapped]

- GIVEN the ordinary offline Cargo sandbox has a wrapper event with a crate name but no unambiguous explicit binding to the selected package/target/features/profile
- WHEN the timeline is produced
- THEN the run and wrapper outcome MUST be retained as unmapped observations
- AND the diagnostic MUST NOT infer dependency edges or unit-specific waits from names, file paths, human progress logs, or nearest timestamps.

#### Scenario: A restored unit did not launch rustc

r[rust_package_planning.wait_diagnostics.identity.cache_hit]

- GIVEN an admitted native unit is restored from cache without compiler execution
- WHEN the session records the unit result
- THEN diagnostics MUST identify the cache disposition and `compiler_ran=false`
- AND MUST NOT synthesize a compiler start/end pair or a zero-duration compilation.

### Requirement: Supported observations only

r[rust_package_planning.wait_diagnostics.supported_events] Diagnostics MUST record actual compiler/process start, termination/status and observed metadata/artifact notifications with producer capability, artifact role, attempt association, and transport-time semantics. Unsupported or absent internal compiler phases MUST be explicitly unavailable, not reconstructed from process lifetime, suffixes, file polling/mtime, exit, silence, or another compiler's protocol. Failed, timed-out, and unfinished attempts MUST remain represented.

#### Scenario: A supported compiler emits a full-metadata notification

r[rust_package_planning.wait_diagnostics.supported_events.full_metadata]

- GIVEN the selected pinned compiler and live collector support structured full-metadata artifact notifications
- WHEN a notification is actually received from an identified compiler attempt before exit
- THEN the diagnostic MUST retain its actual receipt-time offset, capability, full-metadata role and opaque artifact/producer association
- AND MUST distinguish it from early-interface readiness and final linkable-artifact completion.

#### Scenario: Completion-only capture cannot provide live phase timing

r[rust_package_planning.wait_diagnostics.supported_events.unavailable]

- GIVEN the wrapper exposes stderr only after process completion and no early-interface, pause, or resume events are supported
- WHEN the report is produced
- THEN these phase observations and their durations MUST be unavailable with reasons
- AND discovered final outputs MAY be labeled completion-observed artifacts but MUST NOT be assigned retrospective metadata-ready or resume times.

#### Scenario: Timeout occurs before compiler exit is observed

r[rust_package_planning.wait_diagnostics.supported_events.timeout]

- GIVEN a compiler starts and the invocation times out before normal completion
- WHEN the shell requests termination and observes cleanup
- THEN the attempt MUST retain timeout status, observed termination/reap facts and any missing endpoints as censored/unavailable
- AND MUST NOT be removed from the run report or counted as a successful compilation.

### Requirement: Explain distinct observed wait intervals

r[rust_package_planning.wait_diagnostics.wait_classes] The reducer MUST distinguish dependency wait, actual ready-queue delay, and artifact-ready-to-resume delay using exact prerequisite roles and observed endpoints within one invocation. Missing endpoints MUST yield a partial/censored interval or unavailable duration rather than zero or a fabricated phase. Overlapping waits MUST NOT be summed as elapsed build time.

#### Scenario: Dependency-ready and queue events separate two waits

r[rust_package_planning.wait_diagnostics.wait_classes.queue]

- GIVEN a dependent is requested at invocation offset 10 ms, its last required producer artifact is observed ready at 40 ms, and it is explicitly enqueued ready at 45 ms and dispatched at 70 ms
- WHEN the report explains its launch
- THEN it MUST report 30 ms of observed dependency wait and 25 ms of observed ready-queue delay with the producer prerequisite and endpoint references
- AND the 5 ms readiness-to-enqueue gap MUST remain separately unattributed rather than being silently charged to either interval.

#### Scenario: Explicit pause separates file wait from resume delay

r[rust_package_planning.wait_diagnostics.wait_classes.resume]

- GIVEN a supported producer records an identified attempt pausing at 100 ms for two exact artifacts, whose actual readiness notifications arrive at 120 ms and 150 ms, then resuming at 180 ms
- WHEN the report reduces that pause
- THEN it MUST report 50 ms awaiting artifacts and 30 ms post-readiness resume delay with both artifact associations
- AND MUST NOT treat the 80 ms pause as active compiler time.

#### Scenario: Readiness or queue observation is missing

r[rust_package_planning.wait_diagnostics.wait_classes.missing]

- GIVEN a pause and resume are observed but an awaited artifact has no readiness notification, or a launch follows artifact readiness without an explicit enqueue event
- WHEN a wait explanation is requested
- THEN the pause split MUST be unavailable for the missing readiness and any artifact-ready-to-start gap MUST NOT be named ready-queue delay without queue evidence
- AND the report MUST NOT substitute resume time for unknown artifact readiness or assert a complete critical path.

### Requirement: Diagnostic-only schema and invocation-local time

r[rust_package_planning.wait_diagnostics.isolation] Rust timing observations MUST use a separate explicit opt-in diagnostic schema, with monotonic offsets and ordered event identities scoped to one local invocation. They MUST NOT extend or silently repurpose `mantle-build-trace-v1`, merge clocks across hosts/invocations, control scheduling/admission/cache visibility, or add performance data to canonical receipt preimages or cache identities. Collection failure MUST remain separate from ordinary build outcome.

#### Scenario: Diagnostic mode changes only diagnostic evidence

r[rust_package_planning.wait_diagnostics.isolation.enabled]

- GIVEN identical pinned source/toolchain/configuration is built with and without the diagnostic option
- WHEN the enabled run writes `mantle-rust-wait-diagnostics-v1` beside ordinary diagnostics
- THEN existing receipt preimages and cache/admission identities MUST remain independent of timing values and existing causal trace v1 semantics MUST remain unchanged
- AND the collector MUST NOT supply priority, dependency admission, or publication decisions.

#### Scenario: Clock domains cannot be merged

r[rust_package_planning.wait_diagnostics.isolation.clock_domain]

- GIVEN two event records belong to different invocation IDs or hosts even though their monotonic offsets overlap
- WHEN a reducer is asked to explain a wait between them
- THEN it MUST reject cross-domain interval attribution with a deterministic reason
- AND MAY compare completed run durations in separate cohorts but MUST NOT construct a shared timestamp axis or causal edge.

#### Scenario: Diagnostic sink fails while a build proceeds

r[rust_package_planning.wait_diagnostics.isolation.sink_failure]

- GIVEN the opt-in sink becomes unwritable during a successful or failed compiler attempt
- WHEN the ordinary shell completes build execution and cleanup
- THEN it MUST preserve the actual build outcome and independently report unavailable/incomplete diagnostics
- AND MUST NOT suppress compiler failure, authorize incomplete artifacts, change cache visibility, or schedule replacement work because observation failed.

### Requirement: Bounded sanitized diagnostic publication

r[rust_package_planning.wait_diagnostics.bounds] Diagnostic collection, identity correlation, resource samples, retention, and publication MUST enforce documented finite numeric bounds before unbounded allocation. Records MUST sanitize secret/path/control-bearing text before publication and exclude raw argv, environment, source contents, and raw compiler diagnostics. Overflow MUST be explicit and MUST NOT create an unbounded queue or fabricate completeness.

#### Scenario: Sensitive labels are bounded before admission

r[rust_package_planning.wait_diagnostics.bounds.redaction]

- GIVEN an event label includes a host path, secret-bearing token, control byte, and oversized free-form text
- WHEN the event is admitted to the diagnostic sink
- THEN the collector MUST redact sensitive material, escape controls, and enforce the configured text/record/total limits before publishing
- AND safe typed opaque identity fields MAY remain, but raw labels/argv/environment MUST NOT be retained in a secondary diagnostic field.

#### Scenario: Event or correlation budget is exhausted

r[rust_package_planning.wait_diagnostics.bounds.overflow]

- GIVEN an invocation reaches its validated record, byte, or outstanding-correlation ceiling while compiler work continues
- WHEN additional optional observations arrive
- THEN the collector MUST stop retaining those optional observations with bounded incomplete/overflow evidence and retain the terminal run outcome through reserved bounded reporting capacity where the sink remains available
- AND MUST NOT grow an unbounded buffer, claim full coverage, or block compiler cleanup waiting for diagnostic space.

### Requirement: Controlled comparable cohorts with complete run accounting

r[rust_package_planning.wait_diagnostics.cohorts] A comparison report MUST retain every planned/executed run, setup/warm-up failure, timeout and exclusion, and group results by pinned compiler/Cargo, flags, source/lock/input identities, configuration, resources, instrumentation and cache-state policy. Successful comparable cohorts MUST report sample counts, medians and min/max ranges; failed/timed-out durations MUST remain separate censored outcomes. Clean, warm/cache and incremental cohorts MUST remain separate, with alternating treatment order and explicitly recorded controls.

#### Scenario: Comparable successes and failures remain visible

r[rust_package_planning.wait_diagnostics.cohorts.summary]

- GIVEN a predeclared alternating A/B cohort includes successful comparable pairs plus a compiler failure and a timeout
- WHEN the report summarizes elapsed time
- THEN it MUST show the successful sample/pair counts, medians and min/max ranges for each mode, with paired deltas only for successful comparable pairs
- AND MUST retain failure/timeout statuses and durations separately rather than drop them or include them in successful speedup samples.

#### Scenario: Unlike toolchains or cache states are not an optimization A/B

r[rust_package_planning.wait_diagnostics.cohorts.incomparable]

- GIVEN two runs differ in compiler binary/closure, semantic flags, declared inputs, job/resource limits, instrumentation mode, or clean versus incremental/cache state in addition to the named treatment
- WHEN a comparison is requested
- THEN those runs MUST be retained with the mismatch reason and excluded from the same causal A/B summary
- AND unlike native/Cargo lanes MAY be described separately but MUST NOT be called an isolated early-interface speedup.

### Requirement: Repeatable real baselines, resource and failure observations

r[rust_package_planning.wait_diagnostics.repeatability] The measurement recipe MUST exercise real offline small chain and wide fixtures within native admission limits through ordinary supported native local execution and default sandboxed Cargo, without requiring a patched compiler or permanent external benchmark installation. It MUST distinguish serial whole-unit native execution, ordinary full-metadata pipelining, and any future early-interface treatment. It MUST record resource/error coverage, elapsed and process-tree or cgroup CPU/memory metrics, supported paused-resident counts, time-to-first-error and post-error drain, or mark unsupported metrics unavailable. Historical Headstart performance MUST NOT be presented as Mantle evidence.

#### Scenario: A real supported fixture can be repeated

r[rust_package_planning.wait_diagnostics.repeatability.fixtures]

- GIVEN pinned local-source chain and wide fixture manifests, source generation, lockfiles, ordinary supported compiler/toolchain closure and declared resource/cache policies
- WHEN a maintainer follows the recorded recipe twice through native local execution and the default offline Cargo sandbox
- THEN each execution MUST retain its invocation/configuration manifest, actual build/binary outcome, observed event coverage, and cohort accounting
- AND ordinary Cargo full-metadata pipelining MUST be labeled separately from native serial execution and any later early-interface mode, without claiming full Mantle workspace self-hosting.

#### Scenario: Failure timing and memory sampling retain their limits

r[rust_package_planning.wait_diagnostics.repeatability.resources]

- GIVEN a body-error fixture emits a structured error observed live while descendants remain alive, and the configured process-tree or cgroup sampler observes resource usage
- WHEN the invocation fails and the shell reaps the last descendant
- THEN the report MUST retain time-to-first-observed-error, error-to-last-reap drain, CPU user/system accounting, named peak-memory metric, sample interval and coverage, and paused-resident count coverage
- AND sampled peaks MUST be labeled observed lower bounds and overlapping process RSS limitations MUST be stated where applicable.

#### Scenario: Unsupported phases and completion-only errors are not zeros

r[rust_package_planning.wait_diagnostics.repeatability.unsupported]

- GIVEN the ordinary toolchain has no pause/resume protocol and a path exposes compiler errors only at completion
- WHEN the report displays resource and error timing fields
- THEN paused-resident counts and precise first-error timing MUST be unavailable with reasons, with any exit-observed error bound named honestly
- AND MUST NOT claim zero paused memory, immediate error reporting, or historical Headstart savings.

### Requirement: Content-associated edit steps and honest work populations

r[rust_package_planning.wait_diagnostics.edit_steps] The finite measurement manifest MUST associate every case/scenario/step with exact before/after content-set identities, predecessor, session and invocation/attempt identities, independent expected outcomes and pinned clean references. It MUST cover cold load/bootstrap, first edit, subsequent warm edit, no-op/unchanged, cache hit, cache miss, invalidation and error-to-recovery, distinguishing disk result caches, resident query reuse and ordinary Cargo incremental mode. Every selected, eligible, executed, reused, fallback, excluded, failed and unrun outcome MUST remain accounted with scope, denominator, classification relationships and reasons; required blocked, unsupported or incomplete cells MUST be nonpassing. Supported work counters and apply/invalidate/reanalyse/compile/resolve/define/object/link/runnable boundaries MUST retain provider/capability, content/step/session association and actual endpoint coverage. Unsupported counters/phases MUST be unavailable, never zero; zero functions recompiled MUST NOT imply zero latency or no other work. Diagnostics MUST NOT scrape human logs, infer active compiler work from elapsed time, equate source functions with instances or partial fast-tier work with the whole build, or fabricate phase splits. These diagnostics MUST remain independently useful on existing ordinary Cargo/native paths without a resident provider or early metadata, preserving diagnostic-versus-authority isolation.

#### Scenario: Qualified ordinary edits preserve content and actual work

r[rust_package_planning.wait_diagnostics.edit_steps.qualified]

- GIVEN an offline ordinary Cargo/native fixture has a fixed cold, unchanged, body-edit and break/repair sequence with exact content identities, independent expectations and clean references
- WHEN supported collectors observe actual unit launches, cache dispositions and terminal/runnable boundaries for each step
- THEN the report MUST bind each observation to its case, predecessor, content, session and invocation/attempt, retaining observed work populations and their exact scope/denominator
- AND successful current build results and expected real compiler rejections MUST meet the admitted content-specific oracle before comparative interpretation, while unobserved internal queries/phases remain explicitly unavailable.

#### Scenario: Unavailable and mixed-work observations do not become a warm success

r[rust_package_planning.wait_diagnostics.edit_steps.unavailable_mixed]

- GIVEN a fixed ordinary Cargo/native edit manifest includes an unchanged step, a disk-cache miss, an invalidating edit and error-to-recovery, while a later optional provider reports zero functions recompiled for only a supported subset and falls back for other work
- WHEN the consumer inspects the step report with resident-query counts or internal phase boundaries unsupported on the ordinary path and a required cell blocked or unrun
- THEN the report MUST preserve exact before/after content, step/session/attempt associations, separate cache/incremental/reuse dispositions, scoped counts and denominators, fallback and unrun reasons, nonzero observed elapsed samples and unavailable metrics without invented zero values or phase splits
- AND required blocked, unsupported or incomplete cells MUST remain nonpassing and partial-provider coverage MUST NOT be presented as whole-build performance
- AND the recovery MUST reference its independent expectation and clean reference at the recovered content, retaining the preceding compiler failure; timeout, crash or protocol error MUST NOT count as expected compiler rejection
- AND ordinary Cargo/native descriptive reporting MUST NOT require a new warm provider, rustd, Headstart or early-interface events and MUST NOT acquire scheduling, receipt, cache or artifact-publication authority.

### Requirement: Qualified distributions and complete session costs

r[rust_package_planning.wait_diagnostics.session_costs] Reports MUST retain bounded raw samples and first-edit observations, per-scenario successful comparable sample counts, medians and min/max, and a predeclared distribution/percentile estimator and meaningful minimum sample-count policy, or an explicit no-percentiles policy. Below the declared threshold percentiles MUST remain unavailable. Cold/bootstrap, setup, warm-up, restore/reset, failure and censored outcomes MUST remain visible and MUST NOT be removed to invent speedup. Resident, sampled peak and idle footprint reports MUST name metric, process-tree/cgroup scope, interval, exact sampling/lifetime coverage and gaps, idle window and state, and session start/ready/shutdown/reap boundaries; sampled peaks MUST remain lower bounds and missing compiler-memory sampling or shutdown MUST remain unavailable/censored. Correctness/equality qualification for exact contents and independent expectations/clean references, admitted work scope and complete required population accounting MUST gate comparative performance before latency and then footprint. Same-provider cold/warm MUST be a separately named state treatment with pinned provider/binaries/configuration and only the declared state difference; strict same-binary off/on controls MUST remain unchanged. Different-provider/backend/toolchain references MUST declare separately admitted provider identities and supported semantic observation projections and remain descriptive qualification, never same-binary treatment or artifact-byte-equivalence claims.

#### Scenario: Qualified cohorts retain full costs and scoped summaries

r[rust_package_planning.wait_diagnostics.session_costs.qualified]

- GIVEN predeclared same-provider cold/warm cohorts pass independent equality and required work coverage, retain finite raw samples and use a declared distribution policy
- WHEN the report summarizes actual session start, readiness, first/subsequent edits, idle intervals and shutdown/reap observations
- THEN it MUST publish per-scenario comparable counts, medians/min/max and permitted distributions alongside raw/first-edit samples and separate cold/bootstrap/setup/restore costs
- AND memory observations MUST retain metric, process-tree/cgroup scope, sample interval, exact lifetime coverage and gaps, with idle footprint distinct from sampled peak and sampled peaks labelled observed lower bounds
- AND neither the comparison nor its timings MUST change receipt preimages, cache identities, scheduling or final-result authority.

#### Scenario: Hidden bootstrap and sparse resident samples cannot establish speedup

r[rust_package_planning.wait_diagnostics.session_costs.hidden_bootstrap]

- GIVEN a proposed warm summary has first/subsequent edit samples but a failed bootstrap, uncounted warm-up/restore work, a compiler-memory sampling gap and only an idle daemon RSS snapshot, with too few samples for the declared percentile policy
- WHEN a consumer requests a performance comparison against a cold run or different-backend reference without complete equality/work-coverage qualification
- THEN the report MUST retain bootstrap/setup/warm-up/restore outcomes, first-edit and raw samples, per-scenario counts/medians/min/max where comparable, full observed session lifetime and missing/censored regions without subtracting those costs into a speedup
- AND percentiles, unsupported compiler-memory/phase/work values and unsampled peak/idle-lifetime coverage MUST remain explicitly unavailable; an idle snapshot MUST NOT become a session or compiler-tree peak
- AND comparative performance MUST remain unqualified until independent content-specific equality and required work coverage pass, with same-provider cold/warm state treatment separate from strict same-binary off/on
- AND any different-provider reference MUST remain a separately admitted descriptive qualification with explicit identities/projections, without weakening same-binary requirements or asserting artifact-byte equality.
