# Tasks: Watch mode re-evaluates a plan and retracts removed goals

All tasks remain open. Creating this proposal is not producer acceptance.

## Phase 1: Baseline and contract

- [ ] [serial] T1.1 Create an isolated worktree from current `origin/main`. Record current evaluation, conversion, dispatch, and cancellation behavior, plus one repeated-edit observation. r[build_scheduling.watch_plan_assertion]
- [ ] [serial] T1.2 Define the watch event schema and the plan-diff classification over goal identity. r[build_scheduling.watch_plan_assertion]
- [ ] [serial] T1.3 Record the identity-key, cancellation, and failed-re-evaluation decisions in an ADR. r[build_scheduling.watch_error_retention]

## Phase 2: Core and shell

- [ ] [serial] T2.1 Implement pure goal-set diffing: added, retained, retracted, with deterministic ordering and bounded event counts. r[build_scheduling.watch_plan_assertion]
- [ ] [serial] T2.2 Add the opt-in watch shell: source watching, bounded re-evaluation, plan conversion, and diff application to the scheduler. r[build_scheduling.watch_plan_assertion]
- [ ] [serial] T2.3 Cancel in-flight work for retracted goals, release reservations, and record pending cancellation when a build does not stop. r[build_scheduling.watch_retraction_cancellation]
- [ ] [serial] T2.4 Preserve the admitted goal set on evaluation, conversion, or policy failure and report the error. r[build_scheduling.watch_error_retention]

## Phase 3: Fixtures

- [ ] [parallel] T3.1 Add positive fixtures: one-root edit rebuilds one root; added root dispatches only the new root; retained root keeps its completed output. r[build_scheduling.watch_plan_assertion]
- [ ] [parallel] T3.2 Add negative fixtures: delete a root during its build, cancel and record no output; invalid source retains the previous set; rapid edits coalesce without duplicate dispatch. r[build_scheduling.watch_retraction_cancellation]
- [ ] [parallel] T3.3 Add bounds fixtures: event count, live goal count, and re-evaluation rate limits fail closed. r[build_scheduling.watch_error_retention]

## Phase 4: Verification

- [ ] [serial] T4.1 Run the edit, delete-during-build, invalid-source, and coalescing rails before and after the change. Preserve exact results. r[build_scheduling.watch_retraction_cancellation]
- [ ] [serial] T4.2 Run focused tests, formatting, Clippy, `git diff --check`, Cairn validation, Tracey coverage, and the relevant Nix checks. r[build_scheduling.watch_plan_assertion]
- [ ] [serial] T4.3 Sync accepted specs and archive through the isolated branch workflow with retained completion evidence. r[build_scheduling.watch_error_retention]
