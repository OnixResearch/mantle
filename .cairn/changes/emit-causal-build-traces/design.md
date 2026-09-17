# Design: Causal build traces

## Goal and scope

Record why each scheduler action happened, in one bounded artifact, without
changing receipts or scheduling behavior.

## Current behavior

`mantle-evaluation-stream-v1` carries selected-root terminal facts. The build
report carries aggregate outcomes. The transcript records command evidence.
Remote attempts carry a trace context. None of these carries a causal edge
between two scheduler actions.

## Approach review

| Family | Mechanism | Disposition | Required check |
| --- | --- | --- | --- |
| Parse the build log | Reconstruct causality from text | Rejected: unstable and lossy | Cause-chain fixture |
| Extend the aggregate report | Add per-root causes | Rejected: report stays a terminal snapshot, not an action series | Retry fixture |
| Dedicated causal trace | One record per action with a cause identity | Selected direction | Validation and bounds fixtures |
| Full event sourcing of the scheduler | Replace the state machine with an event log | Rejected: larger change than the diagnostic needs | Not blocking |

## Contract and component ownership

- Pure core: record validation, cause-vocabulary validation, chain walk over an
  in-memory record set, bounds checking, and canonical ordering.
- Shell: emission from scheduler calls, file writing, redaction, and
  rendering.
- Boundary: the trace is written beside existing diagnostics and is never read
  by output admission, receipts, or release verification.

## Decisions

### Decision: One record per action, cause by identity

**Choice:** Each record names the action it records and the action that caused
it, using the same record identity.

**Rationale:** An identity edge supports chain walking and rejects dangling
causes. It also survives interleaving from parallel goals.

### Decision: Closed cause vocabulary

**Choice:** Causes come from a fixed list, and an unknown cause fails closed.

**Rationale:** An open vocabulary would degrade into free text. The reviewed
trace schema fixes its cause variants for the same reason.

### Decision: Diagnose, do not promote

**Choice:** Trace records stay outside receipts and are rejected by evidence
validators.

**Rationale:** A causal record describes an observation stream. Evidence must
be reproducible from stored facts, which the trace is not.

## Risks / Trade-offs

- Emission adds an ordering dependency between scheduler events. The shell owns
  it, and a dropped cause is reported rather than invented.
- Long builds produce many records. Bounds fail closed, and the default is a
  capped trace.
- Reviewers may over-read the trace. Non-claims state that completeness is not
  proven.

## Non-Claims

- A trace proves the recorded causal order, not that the order is complete.
- A traced action is not proof that the action succeeded.
