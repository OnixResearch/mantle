# Tasks: Publish live build state to coordination subscribers

All tasks remain open. Creating this proposal is not producer acceptance.

## Phase 1: Contract and provider review

- [ ] [serial] T1.1 Create an isolated worktree from current `origin/main`. Record the current aggregate and NDJSON observation behavior, including one worker-loss observation. r[mantle.build_interchange.live_state_emission]
- [ ] [serial] T1.2 Define the emitted fact and event vocabulary and identities: goal, worker, reservation, and terminal outcome, with size and count bounds. r[mantle.build_interchange.live_state_emission]
- [ ] [serial] T1.3 Evaluate the Molten dataspace contract against the required bounded filters, retraction, and subscription behaviour. Record the decision and the fallback port boundary. r[mantle.coordination_service.live_state_subscription]
- [ ] [serial] T1.4 Record the daemon boundary, best-effort emission, change-driven publication, and restart-retraction decisions in an ADR that extends ADR 0080. r[mantle.coordination_service.daemon_independence]

## Phase 2: Building-plane emission

- [ ] [serial] T2.1 Implement pure fact normalization, fact identity, bounded admission, and the scheduler-state to emit-action mapping. r[mantle.build_interchange.live_state_emission]
- [ ] [serial] T2.2 Emit from a build without blocking, and treat a missing or failed endpoint as a degraded observation event. r[mantle.build_interchange.live_state_emission]
- [ ] [parallel] T2.3 Prove that a build with emission produces identical outputs and receipts to a build without it. r[mantle.coordination_service.daemon_independence]

## Phase 3: Coordination daemon

- [ ] [serial] T3.1 Serve subscriptions with an initial matching set and later publish and retract events, dropping a slow subscriber under a declared bound. r[mantle.coordination_service.live_state_subscription]
- [ ] [serial] T3.2 Retract facts on completion, failure, cancellation, reservation release, worker loss, and daemon restart. r[mantle.coordination_service.retraction_on_owner_stop]
- [ ] [serial] T3.3 Keep the daemon outside store mutation, evidence authoring, and admission, with a narrow CLI surface and lifecycle. r[mantle.coordination_service.daemon_independence]
- [ ] [parallel] T3.4 Add daemon fixtures: subscription order for a multi-root build, retained facts across unrelated transitions, and bounded filter rejection. r[mantle.coordination_service.live_state_subscription]
- [ ] [parallel] T3.5 Add daemon negative fixtures: worker loss, daemon restart, killed subscriber, oversized filter, malformed fact, and slow subscriber drop. r[mantle.coordination_service.retraction_on_owner_stop]

## Phase 4: Verification

- [ ] [serial] T4.1 Prove the plane boundary: no daemon, the build still succeeds; evidence validators reject live facts; the daemon cannot mutate the store. r[mantle.coordination_service.daemon_independence]
- [ ] [serial] T4.2 Run the subscription, retraction, and boundary rails before and after the change. Preserve exact results. r[mantle.coordination_service.retraction_on_owner_stop]
- [ ] [serial] T4.3 Run focused tests, formatting, Clippy, `git diff --check`, Cairn validation, Tracey coverage, and the relevant Nix checks. r[mantle.build_interchange.live_state_emission]
- [ ] [serial] T4.4 Sync accepted specs and archive through the isolated branch workflow with retained completion evidence. r[mantle.coordination_service.daemon_independence]
