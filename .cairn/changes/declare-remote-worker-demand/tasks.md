# Tasks: Declare remote worker demand

Source implementation, positive/negative fixtures, applicable verification,
and scoped production delivery/report observations are individually evidenced
below. Historical before/after rails and lifecycle/archive eligibility remain
open. Passing fixtures and gates are not whole-change acceptance.

## Phase 1: Contract

- [x] [serial] T1.1 Create an isolated worktree from current `origin/main`. Record the current quantified-resource blocker path and one unmet-demand observation. r[build_scheduling.declared_worker_demand]
  `evidence/current-origin-baseline-2026-10-26.json` binds the original
  `db98fd8bc` converted-derivation dispatch observation and exact blocker;
  repeated rounds and concurrency 1/64 create no jobs, leases, or capacity.
  This temporary API instrumentation is not historical pre-feature evidence
  or original CLI ingress/delivery evidence.
- [x] [serial] T1.2 Define the demand fact: requirement vector, job identity, policy identity, blocker reason, requirement identity, and bounds (`crunch-remote-core::demand`; canonical quantities and semantic placement digest). r[build_scheduling.declared_worker_demand]
- [x] [serial] T1.3 Record the restate-only, lifetime-bound, and cannot-grant decisions in Proposed ADR 0127 (not runtime acceptance). r[build_scheduling.demand_without_capacity_inference]

## Phase 2: Core

- [x] [serial] T2.1 Implement pure demand derivation, requirement identity, deduplication, retraction decision, and count bounds (`crates/crunch-remote-core/src/demand.rs` and `src/remote_build/demand.rs`; verification remains T4). r[build_scheduling.declared_worker_demand]
- [x] [serial] T2.2 Prove no inference: demand derivation MUST reject any input path that would convert concurrency, presence, or history into capacity. r[build_scheduling.demand_without_capacity_inference]

## Phase 3: Publication

- [x] [depends:publish-live-build-state-subscriptions] [serial] T3.1 Publish and retract demand facts through the coordination daemon, with the plan result and build report keeping the same blocker when the daemon is absent. r[build_scheduling.declared_worker_demand]
- [x] [serial] T3.2 Keep demand mandatory-visible in the placement report with the same blocker reason and no added capacity claim. r[build_scheduling.demand_without_capacity_inference]

Source adapters exist: coordinator admission records a pending unmet quantified
requirement without adding capacity; scoped live snapshots relay only the
owner's request demands. Production admission keeps its fail-fast blocker and
the preparation shell renders the demand, queues publication, explicitly
retracts `OwnerStopped`, then queues the terminal diff before returning that
same error. This also covers the gateway's shared preparation caller. The
daemon enforces the global demand count bound and rejects publishing a changed
requirement before the old per-job fact retracts. Its absent/unreachable
endpoint never controls admission. The existing publisher is detached and
best-effort: an immediately exiting CLI is not proof of delivered events.
`evidence/actual-derived-demand-2026-10-27.json` retains the real long-lived
gateway/subscriber observation and actual daemon-present/absent CLI controls:
one publish/retract pair per request, unchanged quantified blocker, no jobs or
leases, and no capacity inferred from concurrency 64. T3.1/T3.2 close only this
recorded runtime scope, not reliable delivery or whole-change acceptance.


## Phase 4: Fixtures and verification

- [x] [parallel] T4.1 Add positive fixtures: one unmet demand per requirement, retraction on placement, retraction on cancellation, deduplication across placement rounds, and resolution after a supervisor-registered worker. r[build_scheduling.declared_worker_demand]
- [x] [parallel] T4.2 Add negative fixtures: no demand fact creates a lease, reservation, or placement; demand does not change placement order; demand count bound fails closed. r[build_scheduling.demand_without_capacity_inference]
- [ ] [serial] T4.3 Run the demand, retraction, deduplication, and no-inference rails before and after the change. Preserve exact results. r[build_scheduling.demand_without_capacity_inference]
- [x] [serial] T4.4 Run focused tests, formatting, Clippy, `git diff --check`, Cairn validation, Tracey coverage, and the relevant Nix checks. r[build_scheduling.declared_worker_demand]
- [ ] [serial] T4.5 Sync accepted specs and archive through the isolated branch workflow with retained completion evidence. r[build_scheduling.demand_without_capacity_inference]

## Current evidence boundary (2026-10-26)

`evidence/obligation-reconciliation-2026-10-26.json` maps every T4.1 subcriterion
to exact fixture bodies and retained passing result lines; source inspected
before the negative-test additions matches both retained gate manifests. T4.1
closes fixture addition only. Supervisor response is represented by ordinary
worker registration in the fixture, not by a deployed supervisor process.
Registration with satisfying quantified inventory permits normal admission,
which creates its ordinary resource lease and retracts demand; demand grants
neither that lease nor capacity.

`evidence/negative-controls-verification-2026-10-26.json` records the later
execution of all three added coordinator negatives: presence/concurrency without
inventory, completed history after inventory withdrawal, and actual two-worker
placement/reservation equivalence with and without unrelated unmet demand.
The focused demand rail completed **5 passed, 0 failed, 0 ignored**, including
all three exact named controls on the unchanged post-format source. Reused
core/daemon count-bound passes remain bound to their unchanged source bytes.
T2.2/T4.2 are complete. The earlier handoff and reconciliation are historical
pre-execution records, not evidence that these new controls were previously run.
No production logic changed and no live-delivery claim follows.

The earlier real socket fixture established delivery of a constructed fact,
while the earlier CLI service observation was an empty subscription. The later
`evidence/actual-derived-demand-2026-10-27.json` and its nine retained raw files
record a production-derived fact draining through a real long-lived gateway,
daemon and subscriber, plus mandatory-visible placement/plan/build controls.
The executed rail reports **1 passed, 0 failed, 0 ignored**. Its exact binary
and selected-source bindings are retained; subsequent unrelated source edits
do not turn it into a fresh whole-tree gate.
T4.3's historical-before result and T1.1's isolated current-origin baseline
cannot be supplied by later green gates. The first ordinary gate after the
focused passes failed with retained archive/attestation disk-quota errors under
`/tmp`. A later unchanged-source run with a private scratch-backed `/tmp` passed
ordinary first-party quality, targeted Tiger Style and affected Nix checks.
T4.4 is complete with the captured diff/native gates and criterion mapping.
Declared release-provenance Tracey coverage is not coverage of these demand
requirements. T4.5 remains a separate accepted-spec/archive authorization
boundary. Mantle's scheduling/coordinator live-state maintainers own repeatable
demand rails; Main owns remaining local runtime observations and integration.

## Completed integrated verification (2026-10-26)

The [shared final verification](../reconcile-main-api-lineages-2026-10-17/evidence/continuation-verification-2026-10-26.json)
retains exact commands, exits, selected source/stream BLAKE3, native receipts,
the five-test demand rail and the failed attempts without relabeling them.
Affected Nix Nextest reports **6736 passed, 198 skipped**; skipped cases are not
executed scenario evidence. T1.1 and the scoped T3.1/T3.2 observations are now
recorded separately; T4.3 and T4.5 remain open. No reliable publication,
external provisioning, whole-change acceptance or archive follows.
