# ADR 0099: Bind Rust actions to the executed topology

## Status

Accepted (2026-08-31)

## Context

V96 completed the native Rust topology executor's selected unit set. It observed
5,190 protected exec events across 836 matched actions, with zero unknown,
denied, drifted, or overbound events.

Reconciliation still failed because the action planner created 862 actions from
every derivation in the review graph. The combined topology executor did not
schedule 26 derivations outside its supported execution scope. Those actions
could not produce events, so their required minimum event count remained
unsatisfied.

The review graph is intentionally wider than one execution mode. Action
authority must describe effects that this execution mode can perform, not every
reviewable derivation.

## Decision Drivers

- Keep every executed child process covered by a required action.
- Do not require events from unscheduled units.
- Use the executor's existing deterministic topology selection.
- Keep producer lookup inside the selected execution scope.
- Reject empty, unknown, duplicate, or incomplete selections.
- Do not weaken per-action event minima.

## Decision

Expose the combined topology executor's selected unit IDs as a deterministic,
pure function over the unit derivation graph.

Before installing protected Rust action authority:

1. Derive the exact combined-topology execution unit set.
2. Select only those derivations for action adaptation.
3. Build dependency producer indexes from that selected set.
4. Reject a selected ID that is missing or duplicated in the graph.
5. Reject an empty selection.
6. Plan actions and install ptrace policy from the selected inputs.

The executor continues to derive its order through the same combined topology
function. Unsupported or otherwise unscheduled derivations remain reviewable
graph facts, but they do not become executable actions.

Each selected action still requires at least one matching event. Do not create
synthetic events or mark unscheduled actions optional.

## Alternatives Considered

### Allow zero events for any action

Rejected. This would let an expected compiler or build-script action disappear
without a reconciliation failure.

### Synthesize no-op observations for unscheduled units

Rejected. An observation must describe a kernel execution event.

### Execute all review-graph derivations

Rejected. This changes the bounded topology command and would build unrelated
or unsupported targets.

### Keep all derivations in producer fallback indexes

Rejected. An unscheduled variant must not become executable producer authority
for a selected unit.

## Consequences

- Planned actions match the executor's selected unit set.
- Wider review-graph facts remain available without false missing actions.
- Producer inference cannot escape the selected execution scope.
- Every planned action retains a positive event requirement.
- V96 remains failed evidence. A fresh promoted proof must validate complete
  reconciliation and stage progression.
