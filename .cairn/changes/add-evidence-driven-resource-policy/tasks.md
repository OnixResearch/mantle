# Tasks

## 1. Baseline and data contracts

- [x] [serial] 1.1 Run existing scheduler, resource declaration, lease, report, result-discovery, and accounting tests before core changes. r[remote_builds.resource_observations]
  `evidence/historical-provenance-recovery-2026-10-26.json` binds the
  original September pre-core `bb5aabb2a` command streams, including the
  retained initial failure and three isolated retries. These are recovered
  historical observations, not newly executed tests or a pristine-tree claim.
- [x] [serial] 1.2 Define versioned resource-observation and action-family identity schemas. r[remote_builds.resource_observations]
- [x] [serial] 1.3 Define named sample, age, margin, retry, class, charge, time, and measurement bounds. r[remote_builds.replayable_resource_selection] r[remote_builds.positive_oom_retry]
- [x] [serial] 1.4 Add positive observations and negative missing, stale, incompatible, oversized, and untrusted fixtures. r[remote_builds.resource_observations]

## 2. Build the pure selection core

- [x] [serial] 2.1 Add typed policy inputs, eligible machine classes, quota facts, observations, decisions, and reason codes. r[remote_builds.replayable_resource_selection]
- [x] [serial] 2.2 Enforce declared minima and required architecture, platform, KVM, trust, and isolation features. r[remote_builds.replayable_resource_selection]
- [x] [serial] 2.3 Add deterministic tie-breaking and canonical replay serialization. r[remote_builds.replayable_resource_selection]
- [x] [serial] 2.4 Add positive selection tests and negative no-class, stale-history, quota-denial, and feature-mismatch tests. r[remote_builds.replayable_resource_selection]
- [x] [serial] 2.5 Add property tests for determinism, monotonic minima, stable ordering, and bounded output. r[remote_builds.replayable_resource_selection]

## 3. Add bounded OOM recovery

- [x] [serial] 3.1 Define trusted positive OOM evidence for each supported worker platform. r[remote_builds.positive_oom_retry]
- [x] [serial] 3.2 Implement the pure escalation planner with named retry, class, charge, and wall-time limits. r[remote_builds.positive_oom_retry]
- [x] [serial] 3.3 Create each retry as a new fenced attempt with predecessor linkage. r[remote_builds.positive_oom_retry]
  `evidence/actual-fenced-oom-successor-2026-10-27.json` binds Pueue 74's
  real signed gateway OOM, distinct successor/fence, predecessor-linked quota
  contexts, successful successor settlement and public UsageRead. Disabled,
  non-opted-in and SIGKILL controls do not retry. The executed rail reports
  1 passed, 0 failed, 0 ignored; owned delegated cgroups finish empty.
  This is Linux cgroup-v2 runtime evidence, not complete VM campaign or
  whole-change acceptance.
- [x] [serial] 3.4 Add positive OOM escalation tests and negative arbitrary-failure, ambiguous-exit, exhausted-limit, unavailable-class, and quota-denial tests. r[remote_builds.positive_oom_retry]

## 4. Add accounting and quotas

- [x] [serial] 4.1 Add explicit project and account reservation and reconciliation records. r[remote_builds.usage_reservation_and_reconciliation]
- [x] [serial] 4.2 Make ledger mutations idempotent by attempt identity. r[remote_builds.usage_reservation_and_reconciliation]
- [x] [serial] 4.3 Add bounded API summaries with public units and reason codes. r[remote_builds.usage_reservation_and_reconciliation]
  `evidence/public-usage-api-2026-10-27.json` binds Pueue 72's actual signed
  process-only gateway, fresh UsageRead, public schedule/reasons, retained
  reservation and limit/project/operation-authority denials. Positive gateway
  OOM execution and whole-change acceptance remain separate obligations.
- [x] [serial] 4.4 Add positive charge tests and negative duplicate, cancellation, worker-loss, partial-observation, overflow, and store-failure tests. r[remote_builds.usage_reservation_and_reconciliation]

## 5. Add authorized result sharing

- [x] [serial] 5.1 Add private, project, and explicitly named sharing-scope policy. r[remote_builds.authorized_result_sharing]
- [x] [serial] 5.2 Require request identity, producer signature, policy and platform compatibility, authorized scopes, output identity, and CAS availability. r[remote_builds.authorized_result_sharing]
- [x] [serial] 5.3 Record producer-to-consumer evidence links. r[remote_builds.authorized_result_sharing]
- [x] [serial] 5.4 Add positive same-project and explicit-sharing fixtures plus negative signature, scope, platform, policy, identity, and CAS fixtures. r[remote_builds.authorized_result_sharing]

## 6. Build benchmarks and fault evidence

- [ ] [serial] 6.1 Review `nixbench` workload patterns and licenses before adapting any fixture. r[remote_builds.resource_benchmark_evidence]
- [x] [serial] 6.2 Add fixed-input workloads with public source, license, platform, outcome, and measurement metadata. r[remote_builds.resource_benchmark_evidence]
- [x] [serial] 6.3 Compare observe-only policy decisions with the declared static baseline. r[remote_builds.resource_benchmark_evidence] r[remote_builds.resource_policy_rollout]
  `evidence/benchmark-comparison-6.3-2026-10-27.json` retains the actual
  nine cold builds, nine warm hits and deterministic static/observe-only
  comparisons. All local timer history remains `HistoryUntrusted`; this is
  a fail-closed comparison, not authenticated placement or performance benefit.
- [ ] [serial] 6.4 Add ChaosControl campaigns for OOM, worker loss, duplicate events, accounting interruption, and CAS failure. r[remote_builds.positive_oom_retry] r[remote_builds.usage_reservation_and_reconciliation]
- [x] [serial] 6.5 Export observations and non-claims through Valence and gate them through Cairn. r[remote_builds.resource_benchmark_evidence]
  `evidence/valence-actual-publication-2026-10-27.json` retains Pueue 76's
  independent consumption of the real gateway's process-only publication,
  digest/non-claim/role-promotion negatives, and native validation plus all
  three gates. Identity/linkage and structural PASS do not establish worker
  authority, positive OOM, complete campaigns or whole-change acceptance.

## 7. Roll out and validate

- [x] [serial] 7.1 Add observe-only mode and project opt-in controls. r[remote_builds.resource_policy_rollout]
- [x] [serial] 7.2 Add independent rollback controls for selection, retry, quota enforcement, and sharing. r[remote_builds.resource_policy_rollout]
- [x] [serial] 7.3 Run the canonical first-party format check plus format checks for affected first-party packages outside its declared scope. Vendored snapshots are excluded by repository policy. r[remote_builds.replayable_resource_selection]
- [ ] [serial] 7.4 Run focused positive, negative, replay, property, benchmark, and fault tests. r[remote_builds.resource_benchmark_evidence]
- [x] [serial] 7.5 Run `scripts/check-first-party-quality.sh`: GCC4.0 bridge, scoped formatting, vendor-excluded strict Clippy, and serialized workspace lib/tests. Preserve failures and capability skips. r[remote_builds.replayable_resource_selection]
- [x] [serial] 7.6 Run the relevant Nix flake checks. r[remote_builds.resource_benchmark_evidence]
- [x] [serial] 7.7 Run Cairn validation, requirement coverage, design gate, and tasks gate. r[remote_builds.resource_policy_rollout]

## Source repair handoff (2026-10-23; not gate acceptance)

The archived `head-crunch-bin-16-isolated.log:33-46` under
`2026-10-22-schedule-tigerstyle-hardening-backlog/evidence/v1-gates-2026-10-22`
records the three resource-host fixtures failing quota-store access. Their
directories now explicitly satisfy the production 0700 requirement; production
access policy is not weakened. Host observations accept explicit shell timestamps,
and fixture clocks no longer consult ambient coordinator time. Added negative
coverage requires stale-fence/cross-scope mutations to preserve the entire quota
snapshot. The pure ledger rejects a repeated usage digest with changed observation
content; an exact repeat remains idempotent and durable across reopening.

The local benchmark driver is migrated from stale observation v1 fields to v2.
It records actual process-resource measurements without promoting RSS, output
file bytes, absent queue/transfer facts or a local timer into authenticated worker
authority. Observe-only replay must reject those records as `HistoryUntrusted`
and retain static placement. Authentic historical-placement performance still
requires real authenticated worker dimensions, not a rewritten acceptance claim.

The design's focused verification handoff lists exact test/benchmark commands and
external prerequisites. No builds, tests, benchmarks, delegated-OOM runs, VM
campaigns, replay verdicts or lifecycle gates were run for these edits. All runtime
and evidence tasks remain unchecked; no archive/sync or accepted-spec update is
authorized by the source handoff alone.

## Initial bounded outcome (2026-10-26)

The [local-obligations evidence](evidence/local-obligations-2026-10-26.json)
maps every authored criterion to exact source ranges, source/cohort/gate BLAKE3
bindings, named retained passing results, and remaining prerequisites. The
historical source-only handoff above is preserved; it is not the basis for these
current checkmarks. Twenty-two individually supported implementation, fixture
and retained-gate criteria are complete; eleven composite or unobserved criteria
remain unchecked. This is not whole-change or rollout acceptance.

Eight real invocations of the existing immutable production replay executable
retained literal supplied inputs, outputs, exits and stream digests. Identical
inputs repeat identically; supplied positive history can select a larger class
for comparison while observe-only dispatch remains static. Untrusted/stale
history is excluded, independent feature mismatch and quota denial fail closed,
and an unknown input field exits 2. Supplied `trusted_collector` facts do not
authenticate a worker. This did not execute the signed benchmark driver, place
real historical work, observe a cgroup OOM, or dispatch/settle an OOM successor.

Named selector, property, quota-store, sharing and resource-host fixture passes
come from the retained v13 ordinary-first-party stdout, not the later flake
Nextest aggregate. At that capture, selected source bytes matched both gate
manifests and the replay cohort pin manifest. V13 quality and extra format
steps passed, but its failed flake step remains failed; the separate later
full-flake recovery (Pueue 40, exit 0) supports 7.6. No build, test, formatter,
linter, campaign, Cairn or Nix gate was launched by this continuation worker.

The resource-policy, quota-store and gateway maintainers own the repeatable
local capability. Main owns fresh document/lifecycle validation after integration;
7.7 remains unchecked pending its entire authored gate set. Historical-before
evidence, all supported-platform OOM contracts, actual fenced OOM successors,
the full negative retry matrix, bounded public usage API proof, license-review
chronology, real benchmarks, complete campaign acceptance, actual Valence export
and external consumer/adoption prerequisites remain explicit in the JSON.
Declared release-provenance 155/155 marker coverage is not all-open-change
coverage. That bounded replay continuation changed no source, accepted spec,
archive, release or VCS state.

## Completed escalation fixture extension (2026-10-26)

The [fixture handoff](evidence/escalation-fixture-handoff-2026-10-26.json)
records the subsequent test-only refactoring of
`distributed::resource_policy::tests::oom_retry_needs_positive_current_fenced_evidence_and_all_limits`
and the new focused test
`distributed::resource_policy::tests::oom_retry_rejects_nonfailed_attempt_and_unavailable_larger_class`.
A shared request fixture keeps every new/changed function under 70 lines without
adding an allow. The new test requires `NotPositiveOom` for a nonfailed attempt
despite supplied positive evidence and `ClassLimit` when the only larger class is
unavailable despite sufficient quota. The original test explicitly keeps
`failed=true` with category `None` for arbitrary non-OOM failure and `Unknown` for
ambiguous termination; these are distinct from the new nonfailed-attempt case.
Existing fence, retry, class-order, charge, wall and quota negatives are preserved,
not duplicated. No production semantic/API change was made.

The later [fixture verification](evidence/escalation-fixture-verification-2026-10-26.json)
binds both exact named passes to the unchanged post-extension source and raw
streams: **10 passed, 0 failed, 0 ignored** in the focused resource-policy rail.
Task 3.4 is now complete. The initial 22-task record above remains historical;
this individually supported extension brings implementation/fixture/gate tasks
to 23 complete, with ten composite or unobserved tasks still open.

The subsequent ordinary gate failed after the focused passes; its raw output
retains archive/attestation `Disk quota exceeded` errors below hard-coded `/tmp`
roots. This is not a final integrated-gate pass. Main owns the remaining gates.
Actual platform OOM, successor dispatch/settlement, API 4.3, benchmarks,
campaign/export and external acceptance boundaries remain open.

## Native lifecycle verification (2026-10-26)

The [native verification receipt](evidence/native-lifecycle-verification-2026-10-26.json)
retains eleven successful commands: root validation, declared Tracey coverage,
and proposal/design/tasks gates for the three touched active changes. Task 7.7
is complete, bringing individually supported tasks to 24 complete and nine open.
The declared profile references 155/155 release-provenance requirements only;
it is not scenario coverage for this resource policy or every open change.
Criterion-specific fixture/replay records retain their own scope and gaps.
Structural native gates do not grant semantic acceptance, rollout, accepted-spec
sync or archive authority. Final product-gate recovery remains separate.

## Completed integrated product gates (2026-10-26)

The [shared final verification](../reconcile-main-api-lineages-2026-10-17/evidence/continuation-verification-2026-10-26.json)
retains the later unchanged-selected-source ordinary first-party, targeted Tiger
Style and affected Nix fmt/clippy/Nextest passes. A private scratch-backed `/tmp`
restored required filesystem admission without deleting shared files or changing
production behavior. Earlier permission/quota failures remain failed records.
The Nix result is **6736 passed, 198 skipped**, not evidence for skipped cases
or a new full-flake/release claim. The nine remaining authored tasks and their
runtime, historical-before and external-acceptance boundaries stay open.

## Signed kernel OOM observation (2026-10-27)

`evidence/signed-kernel-oom-2026-10-27.json` retains the exact successful
dedicated-collector command and result: **1 passed, 0 failed, 0 ignored**.
The Linux cgroup-v2 collector observes a real OOM kill, signs the current
attempt/request/result facts, rejects a changed result digest, and distinguishes
an ordinary SIGKILL from positive OOM evidence. The larger resource-host
successor completes and settles under a new attempt and advanced fence.
Linux dedicated cgroup-v2 is the supported positive OOM backend; absent or
unsupported collectors provide no positive OOM authority.

This closes the evidence contract in 3.1, not gateway execution in 3.3. The
enclosing v3 run remains failed because its gateway fixtures imported zero
signed inputs. The shared fixture now selects the declared store basename and
requires one rejection-free import; its fresh gateway/demand observations are
pending. No public usage, campaign, acceptance or archive claim follows from
the independent collector result. The remote worker/resource-host maintainers
own this opt-in delegated-cgroup rail.
