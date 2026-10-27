# Tasks: Watch mode re-evaluates a plan and retracts removed goals

The source-only finishing pass of 2026-10-23 added fail-closed scheduler
rejection recovery, accurate pending-task reporting, explicit rate rejection,
stable human run identity, behavioral regressions, and a real CLI smoke rail.
The subsequent local rail passed in Pueue task 37 on the retained immutable
executable: edits, added/retained roots, invalid-import retention, coalescing,
delete-during-build cancellation, no late output, and graceful shutdown were
observed. Evidence: `evidence/final-immutable-watch-runtime-20261023.json`.
T2.2/T3.1/T3.2 are accepted only for that scoped implementation/runtime subset.
The follow-on `evidence/retention-and-bounds-2026-10-26.json` supports T2.3's
controlled Worker teardown deadline, T2.4's evaluation/conversion/policy
retention, and T3.3's executed core boundary fixtures. Pre-change runtime
obligations and lifecycle closure remain separate. T4.2's executed checks are
recorded in `evidence/verification-2026-10-26.json`.

## Phase 1: Baseline and contract

- [x] [serial] T1.1 Create an isolated worktree from current `origin/main`. Record current evaluation, conversion, dispatch, and cancellation behavior, plus one repeated-edit observation. r[build_scheduling.watch_plan_assertion]
- [x] [serial] T1.2 Define the watch event schema and the plan-diff classification over goal identity. r[build_scheduling.watch_plan_assertion]
- [x] [serial] T1.3 Record the identity-key, cancellation, and failed-re-evaluation decisions in an ADR. r[build_scheduling.watch_error_retention]

## Phase 2: Core and shell

- [x] [serial] T2.1 Implement pure goal-set diffing: added, retained, retracted, with deterministic ordering and bounded event counts. r[build_scheduling.watch_plan_assertion]
- [x] [serial] T2.2 Add the opt-in watch shell: source watching, bounded re-evaluation, plan conversion, and diff application to the scheduler. r[build_scheduling.watch_plan_assertion]
- [x] [serial] T2.3 Cancel in-flight work for retracted goals, release reservations, and record pending cancellation when a build does not stop. r[build_scheduling.watch_retraction_cancellation]
- [x] [serial] T2.4 Preserve the admitted goal set on evaluation, conversion, or policy failure and report the error. r[build_scheduling.watch_error_retention]

## Phase 3: Fixtures

- [x] [parallel] T3.1 Add positive fixtures: one-root edit rebuilds one root; added root dispatches only the new root; retained root keeps its completed output. r[build_scheduling.watch_plan_assertion]
- [x] [parallel] T3.2 Add negative fixtures: delete a root during its build, cancel and record no output; invalid source retains the previous set; rapid edits coalesce without duplicate dispatch. r[build_scheduling.watch_retraction_cancellation]
- [x] [parallel] T3.3 Add bounds fixtures: event count, live goal count, and re-evaluation rate limits fail closed. r[build_scheduling.watch_error_retention]

## Phase 4: Verification

- [ ] [serial] T4.1 Run the edit, delete-during-build, invalid-source, and coalescing rails before and after the change. Preserve exact results. r[build_scheduling.watch_retraction_cancellation]
  - Retrospective reconciliation: `evidence/before-after-observations-2026-10-27.json` binds the genuine historical binary's `--watch` rejection (product exit 2, same-driver exit 1, no transitions) and the immutable integrated v2 binary's six observed transitions, including all four requested after scenarios and 21-second teardown/no-late-output observation. Before scenarios were not reached, so the literal four-scenario before obligation remains unresolved; no historical pass or T4.1 acceptance is claimed. The retained v3 request has no observed result and cannot establish latest-source coverage.
- [x] [serial] T4.2 Run focused tests, formatting, Clippy, `git diff --check`, Cairn validation, Tracey coverage, and the relevant Nix checks. r[build_scheduling.watch_plan_assertion]
- [ ] [serial] T4.3 Sync accepted specs and archive through the isolated branch workflow with retained completion evidence. r[build_scheduling.watch_error_retention]
