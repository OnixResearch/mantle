# Design: Watch mode re-evaluates a plan and retracts removed goals

## Goal and scope

One long-running process holds an admitted goal set derived from a Nickel
source. A source change produces a new goal set. The difference between the
old and new sets is applied as assertions and retractions. Retraction cancels
in-flight work.

## Current behavior

`mantle build --watch` is implemented as an opt-in, persistent local Worker.
The shell polls the selected source and parsed transitive imports, buffers
complete evaluation/conversion, and stages an identity diff. A one-use token
gates dispatch after stale sandbox teardown and a fresh source snapshot check.
Ordinary `mantle build` remains one-shot.

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

The in-process `WatchNotice` channel carries bounded goal transitions with
one session-stable `GoalDiff.run_identity`, a monotonic generation, and ordered
`added`, `retained`, or `retracted` identities. The CLI prints that identity on
admission and rejection lines. Watch output is human-readable; it is **not**
an implemented `mantle-watch-transition-v1` NDJSON contract and does not change
one-shot JSON or the evaluation stream. Pending cancellation and settled root
outcomes are separate notices. There is no separate fabricated `cancelled`
success event: retraction is admitted only after the worker's teardown fence.

Each admitted generation has at most 10,000 goals, with at most 20,000 sorted
diff events. The scheduler independently bounds the complete dependency closure
and staged registry capacity before mutating live roots. A scheduler capacity,
missing-input, or static-binding-policy rejection restores the last committed
generation rather than terminating the session. Duplicate identities coalesce.
The shell polls once per second and allows at most 60 actual evaluations per
rolling 60-second window; exhaustion reports one bounded rejection until the
window admits another attempt. The live set is unchanged on rejected input.
Snapshots admit at most 256 parsed source files, 2 MiB per file, 16 MiB total.
Events are coordination, not build success or release evidence.

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

## Current fixtures and required fresh execution

The existing pure `crunch-watch-core` fixtures cover deterministic edit/add
classification, superseded candidates, invalid input retention, exact event
limits, raw goal-input limits, and evaluation-attempt limits. Worker fixtures
exercise completed signed outputs across edit/add/dedup generations, malformed
scheduler-candidate recovery without redispatch, input policy retention,
transitive live-goal rejection, superseded commit tokens, and a running-root
deletion whose successful late result cannot receive signed PathInfo. The
deletion fixture deliberately holds teardown past the five-second reporting
deadline and requires `PendingCancellation` to name only running tasks, not
queued stale goals. These are fixtures to run, not fresh execution evidence.

After rebuilding the actual CLI from the final integrated source, run:

```sh
cargo test -p crunch-watch-core
cargo test -p crunch-build --lib watch_
cargo test -p crunch-build --lib superseded_candidate_token
cargo test -p crunch-pipeline --lib watch::tests
python3 scripts/watch-goal-retraction-smoke.py \
  --mantle /absolute/path/to/fresh/mantle \
  --scratch /tmp/mantle-watch-goal-retraction-UNIQUE
```

The scratch directory must not exist; its basename uses letters, digits,
hyphens, or underscores. The rail requires Linux `/proc`, working Bubblewrap
with pidfds, and the ordinary `/bin/sh`/`/bin/busybox` sandbox tool mapping.
It performs real edit, addition, invalid imported source, coalesced edits, and
delete-during-build operations in **one** watch process, without invoking Cargo
or rebuilding Mantle. Failure retains logs and does not silently force-kill a
worker still owning sandbox teardown. Only a successfully executed rail writes
`result.json` with `status: passed`; no receipt is present merely from adding it.

Expected retained artifacts are `command.json`, `watch.stdout.log`,
`watch.stderr.log`, `owned-slow-processes.json`, `result.json`, all edited Nickel
files, the isolated store, and state. For scratch basename `RUN`, the physical
store contains exactly one output each named `watch-RUN-keep`,
`watch-RUN-moving-v1`, `watch-RUN-moving-v2`, `watch-RUN-added`, and
`watch-RUN-rapid-20` (each with its store digest prefix). Retraction does not
garbage-collect already completed historical outputs. It must contain no
`watch-RUN-rapid-1` through `watch-RUN-rapid-19` and no `watch-RUN-slow` output.
The `keep` artifact's bytes, inode, and mtime must remain unchanged.

Expected notices: initial two-root settlement; one-root replacement with one
`added`, one `retracted`, one `retained`; addition with one `added` and two
`retained`; invalid import with `edit rejected` and no retraction; one coalesced
replacement; slow-root retraction followed by three-root settlement only after
its captured child identities stop. Every admission/rejection reports the same
run identity. The real deletion waits beyond the payload's twenty-second late
write opportunity and rejects any physical or reported slow-root output. A fast
real teardown need not emit pending cancellation; the gated Worker fixture
covers the deadline case. Neither rail nor event proves reproducibility,
source authenticity, or release eligibility. No commands in this section were
run during the source-only finishing pass.

