# Tasks: Declare remote worker demand

Source implementation is recorded below; runtime rails, integration acceptance,
and archive eligibility remain open. No new execution result is claimed here.

## Phase 1: Contract

- [ ] [serial] T1.1 Create an isolated worktree from current `origin/main`. Record the current quantified-resource blocker path and one unmet-demand observation. r[build_scheduling.declared_worker_demand]
- [x] [serial] T1.2 Define the demand fact: requirement vector, job identity, policy identity, blocker reason, requirement identity, and bounds (`crunch-remote-core::demand`; canonical quantities and semantic placement digest). r[build_scheduling.declared_worker_demand]
- [x] [serial] T1.3 Record the restate-only, lifetime-bound, and cannot-grant decisions in Proposed ADR 0127 (not runtime acceptance). r[build_scheduling.demand_without_capacity_inference]

## Phase 2: Core

- [x] [serial] T2.1 Implement pure demand derivation, requirement identity, deduplication, retraction decision, and count bounds (`crates/crunch-remote-core/src/demand.rs` and `src/remote_build/demand.rs`; verification remains T4). r[build_scheduling.declared_worker_demand]
- [ ] [serial] T2.2 Prove no inference: demand derivation MUST reject any input path that would convert concurrency, presence, or history into capacity. r[build_scheduling.demand_without_capacity_inference]

## Phase 3: Publication

- [ ] [depends:publish-live-build-state-subscriptions] [serial] T3.1 Publish and retract demand facts through the coordination daemon, with the plan result and build report keeping the same blocker when the daemon is absent. r[build_scheduling.declared_worker_demand]
- [ ] [serial] T3.2 Keep demand mandatory-visible in the placement report with the same blocker reason and no added capacity claim. r[build_scheduling.demand_without_capacity_inference]

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
Positive event evidence must use a real long-lived adapter/subscriber where
queued publications can drain; the absent-daemon CLI remains a negative
control. T3 remains unchecked until those observations are captured.


## Phase 4: Fixtures and verification

- [ ] [parallel] T4.1 Add positive fixtures: one unmet demand per requirement, retraction on placement, retraction on cancellation, deduplication across placement rounds, and resolution after a supervisor-registered worker. r[build_scheduling.declared_worker_demand]
- [ ] [parallel] T4.2 Add negative fixtures: no demand fact creates a lease, reservation, or placement; demand does not change placement order; demand count bound fails closed. r[build_scheduling.demand_without_capacity_inference]
- [ ] [serial] T4.3 Run the demand, retraction, deduplication, and no-inference rails before and after the change. Preserve exact results. r[build_scheduling.demand_without_capacity_inference]
- [ ] [serial] T4.4 Run focused tests, formatting, Clippy, `git diff --check`, Cairn validation, Tracey coverage, and the relevant Nix checks. r[build_scheduling.declared_worker_demand]
- [ ] [serial] T4.5 Sync accepted specs and archive through the isolated branch workflow with retained completion evidence. r[build_scheduling.demand_without_capacity_inference]
