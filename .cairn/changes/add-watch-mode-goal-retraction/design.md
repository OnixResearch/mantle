# Design: Watch mode re-evaluates a plan and retracts removed goals

## Goal and scope

One long-running process holds an admitted goal set derived from a Nickel
source. A source change produces a new goal set. The difference between the
old and new sets is applied as assertions and retractions. Retraction cancels
in-flight work.

## Current behavior

`mantle build` evaluates, converts, dispatches, and exits. The scheduler
deduplicates goals by store identity and tracks waiters. Existing cooperative
cancellation covers selected-root evaluation, not teardown of a running
sandbox child. No component diffs two plan generations or retracts a goal
after dispatch.

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

## Baseline observation (2026-10-01)

An isolated worktree was created at `origin/main` (`da00f5842`). The existing
shared development binary (`mantle 0.1.0`, not rebuilt from the isolated tree)
evaluated an external Nickel source with `watch-baseline-first` and
`echo first > $out`. After editing that same source, a second explicit
`mantle eval` process returned `watch-baseline-second` and
`echo second > $out`. No live watch process or retraction event existed in
either invocation; this is a repeated-edit baseline observation, not a
build/cancellation or isolated-commit executable proof.

The same shared binary subsequently built `examples/hello.ncl` with
`--no-substitute` in an isolated scratch store/state after creating the
physical store directory. It returned
`/home/brittonr/scratch/mantle-watch-baseline-store-20260930/z2y3k7267f9kbpfyw9zw0d1adjcn6z62-hello`.
That one-shot success establishes sandbox feasibility on this host, not
delete-during-build cancellation or a watch-mode result.

In the isolated source, `crunch-pipeline::build_with_stream_sender` opens an
evaluation session and store, then joins root streaming with a
`Worker::run_streaming` call. `GoalRegistry` keys goals by absolute derivation
store path and deduplicates `get_or_insert`; `Worker` uses a `JoinSet` and
semaphore to dispatch ready goals. Neither a committed next-generation goal
set nor a per-goal watch retraction is present at this boundary. The existing
`EvaluationCancellation` handles evaluation-stream interruption; it is not
proof of running sandbox-build cancellation.

## Watch transition event boundary

The opt-in watch channel uses `mantle-watch-transition-v1` NDJSON with one
object per event, carrying `schema`, a session-stable `run_identity`, bounded
`generation` and `sequence`, and `kind`. Goal events (`added`, `retained`,
`retracted`, `cancelled`, `pending-cancellation`) carry the scheduler
`goal_identity`; a `rejected` generation carries a bounded, redacted
diagnostic and no goal identity. Each admitted generation has at most 10,000
live goals and at most 20,000 diff events; a caller-specified event budget
may be tighter. Identical goal identities coalesce before event emission.
The shell enforces a bounded re-evaluation window (at most 60 attempts per
declared window), timestamps/coalesces observed source and declared-import
changes, and guards the output writer against partial event lines. An event
count, goal count, or evaluation-rate violation rejects the new generation
without retracting the last admitted set. Events record coordination, not
build success or release evidence.

## Decisions

### Decision: Goal identity is the diff key

**Choice:** Diff two plan generations by goal identity, not by label or order.

**Rationale:** The scheduler already deduplicates by identity. Reusing that key
keeps retained work stable across edits and makes the diff deterministic.

### Decision: Retraction cancels, but never fabricates success

**Choice:** A retracted goal requests cancellation of its in-flight build.
Its reservation is released only after actual sandbox-child and descendant
teardown; while teardown is pending it cannot admit a late success.

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
