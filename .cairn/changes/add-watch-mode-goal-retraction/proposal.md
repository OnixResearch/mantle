# Proposal: Watch mode re-evaluates a plan and retracts removed goals

## Why

Mantle evaluates a Nickel source per command. After an edit, the operator runs
the command again and the process selects its roots from scratch. There is no
mode that keeps a live goal set, diffs it after a change, and cancels work that
no longer exists.

The Synit configuration watcher reloads a changed script and discards all state
derived from the previous version of that file
(`~/.local/share/mantle-references/synit-book/pages/17-operation__builtin__config-watcher.md`,
reviewed in `docs/synit-application-notes.md`). The same semantics fit a build
plan: a goal that the current source no longer declares must not keep building.

Mantle already has the pieces: bounded evaluation, plan conversion, a
deduplicating goal scheduler, and cooperative cancellation. Watch mode composes
them instead of adding a second scheduler.

ADR 0080 keeps watch mode in the building plane. Its diff events are
coordination output that the daemon may serve.

## What Changes

- Add an opt-in watch mode that re-evaluates the selected source after a
  change and computes the admitted goal set for the current source.
  r[build_scheduling.watch_plan_assertion]
- Diff the new goal set against the live goal set by goal identity. New goals
  are added. Goals that the new source does not declare are retracted.
  r[build_scheduling.watch_plan_assertion]
- Cancel in-flight work for a retracted goal and release its reservation. A
  retracted goal MUST NOT be recorded as a successful output.
  r[build_scheduling.watch_retraction_cancellation]
- On an evaluation, conversion, or policy error, keep the previously admitted
  goal set running and report the error. A failed re-evaluation MUST NOT
  retract valid goals.
  r[build_scheduling.watch_error_retention]
- Report watch transitions as bounded events: added, retracted, cancelled,
  retained, and rejected, with a stable run identity.

## Impact

- **Immediate consumer**: the developer loop for `mantle build <source>` and
  the examples workflow.
- **Immediate outcome**: an edit rebuilds only the affected goals, and a
  deleted root stops building immediately.
- **Durable capability**: a live plan-diff boundary that the later
  coordination surface can publish.
- **Maintenance owner**: Mantle scheduling owner, covering evaluation reuse and
  goal lifecycle.
- **Repeatability evidence**: edit fixtures for one root and two roots, a
  delete-during-build cancellation fixture, an invalid-source retention
  fixture, and a rapid-edit coalescing fixture.
- **Compatibility**: watch mode is opt-in. Existing one-shot commands keep
  their behavior and output contracts.

## Scope

The change covers watch-mode evaluation, plan diffing, goal assertion and
retraction, cancellation, error retention, event reporting, and fixtures.

## Out of Scope

- Replacing the one-shot evaluation path or the NDJSON stream contract.
- Watching files other than the selected source and its declared imports.
- Treating watch events as build evidence or release evidence.
- Changing GC, trust, or output admission rules for watched builds.

## Success Criteria

- Editing one root rebuilds only that root.
- Deleting a root during its build cancels the build and records no output.
- A syntax error in the source leaves the previous goal set intact.
- Rapid edits coalesce and never dispatch the same goal twice.
