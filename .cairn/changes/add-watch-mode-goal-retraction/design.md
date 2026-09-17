# Design: Watch mode re-evaluates a plan and retracts removed goals

## Goal and scope

One long-running process holds an admitted goal set derived from a Nickel
source. A source change produces a new goal set. The difference between the
old and new sets is applied as assertions and retractions. Retraction cancels
in-flight work.

## Current behavior

`mantle build` evaluates, converts, dispatches, and exits. The scheduler
already deduplicates goals by store identity, tracks waiters, and supports
cooperative cancellation. No component diffs two plan generations or retracts
a goal after dispatch.

## Approach review

| Family | Mechanism | Disposition | Required check |
| --- | --- | --- | --- |
| Re-run the command on change | External shell loop | Rejected: no cancellation, duplicate dispatch, no diff facts | Rapid-edit fixture |
| Rebuild the whole goal set per change | Stop all, restart all | Rejected: loses completed work and reruns unrelated roots | One-root edit fixture |
| Plan diff with retraction | Assert new goals, retract removed goals, cancel on retraction | Selected direction | Edit, delete-during-build, and error fixtures |
| Depend on an external build daemon | Delegate watch to another process | Rejected for this change: Mantle owns its scheduler | Not blocking |

## Contract and component ownership

- Pure core: goal-set normalization, diff classification (added, retained,
  retracted), retraction ordering, and cancellation decision over in-memory
  goal facts.
- Shell: file watching, re-evaluation, plan conversion, scheduler calls,
  cancellation, and event rendering.
- Boundary: watch mode uses the same evaluation budgets, policy, and output
  admission as one-shot builds. It does not weaken them.

## Decisions

### Decision: Goal identity is the diff key

**Choice:** Diff two plan generations by goal identity, not by label or order.

**Rationale:** The scheduler already deduplicates by identity. Reusing that key
keeps retained work stable across edits and makes the diff deterministic.

### Decision: Retraction cancels, but never fabricates success

**Choice:** A retracted goal cancels its in-flight build and releases its
reservation. It is not recorded as succeeded.

**Rationale:** A goal that no longer exists has no declared output. Recording
it would create an output with no current requirement.

### Decision: Failed re-evaluation preserves the admitted set

**Choice:** An evaluation, conversion, or policy error keeps the previous goal
set running and reports the error.

**Rationale:** A typo must not tear down a working build. This matches the
Synit watcher rule that state is discarded only when the changed file is
reloaded successfully.

## Risks / Trade-offs

- A watched build holds process state longer. Bounds on live goals, events, and
  re-evaluation rate carry over from the existing limits.
- File-system watchers are platform-specific. The shell owns them, and the
  core stays pure and testable without a watcher.
- Cancellation timing is cooperative. A build that ignores cancellation is
  reported as pending cancellation rather than silently retained.

## Non-Claims

- Watch events are coordination facts, not build evidence.
- A retained goal is not proof that its output is current after the edit; the
  normal identity checks still decide reuse.
