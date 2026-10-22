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

### Record format and admission

The opt-in diagnostic is canonical UTF-8 JSON:
`{"schema":"mantle-build-trace-v1","records":[...]}`. A record contains
`action_id` (zero-based `u32`), `goal_blake3` (nullable 64 lowercase hex
characters, BLAKE3 of the full logical derivation key), `kind` (a closed
kebab-case action), `cause` (a closed kebab-case cause), `caused_by` (an
`action_id`), and a nullable redacted `diagnostic`. The sole action zero is
`external-trigger` with a self-cause; every later record names an earlier
existing action with a typed compatible kind. Serialized records are canonical
in action ID order, not by goal or wall-clock time. An interested second root
does not rewrite a shared dependency's first triggering cause.

A same-goal cache, dispatch, retry, cancellation, or cleanup edge retains
the parent's redacted goal identity. A cross-goal failure edge is legal only
when an earlier `dependency-ready` action recorded that direct
dependent/dependency pair; a transitive leaf failure instead walks each
immediately failed dependency. Merely sharing a parent kind or appearing
earlier in an interleaved trace does not establish a causal dependency.

Ordinary local `mantle build --causal-trace` has no retry or cancellation
transition. Watch-mode cancellation is a separate, untraced flow. Those kinds
exist in the closed vocabulary and collector fixtures, but production emission
records only transitions actually observed on the traced path.

Closed causes: `root-requirement`, `dependency-ready`, `dispatch`,
`cache-decision`, `retry`, `cancellation`, `cleanup`, `external-trigger`.
Closed actions: `external-trigger`, `root-requirement`, `dependency-ready`,
`cache-check`, `cache-hit`, `cache-miss`, `dispatch`, `retry`, `succeeded`,
`failed`, `cancellation`, `cleanup`. The core rejects unknown and missing
fields, unknown causes/kinds, absent or forward-pointing causes, invalid
action/cause/parent-kind combinations, non-canonical order, invalid digest
identities, oversized records, and unredacted diagnostics. The shell caps
input **before parsing** and rechecks canonical serialized bytes. Hard caps:
16,384 action records, 1,024 encoded bytes per record, 4 MiB per trace, and
256 UTF-8 bytes per diagnostic. Diagnostic token redaction uses the
evaluation-stream rules for absolute paths, credential-shaped names, and
standalone digests before persistence; read-time validation fails closed.
Trace persistence is a separate mode-0600 file under the state diagnostic
`logs/` directory. Its bytes are not embedded in the aggregate report or any
receipt.
Creation is exclusive: an existing diagnostic is never overwritten, including
when a later build reuses a process ID; a numbered suffix keeps both artifacts.

If strict store preflight rejects a fallback before a Worker goal exists, the
CLI retains the original failure report and exposes typed `WorkerNotStarted`
trace unavailability on stderr. No root/request-bound action can be observed
there, so no seed-only or synthetic success/failure trace is persisted.

## Risks / Trade-offs

- Emission adds an ordering dependency between scheduler events. The shell owns
  it, and a dropped cause is reported rather than invented.
- Long builds produce many records. Bounds fail closed, and the default is a
  capped trace.
- Reviewers may over-read the trace. Non-claims state that completeness is not
  proven.

## Non-Claims

- A trace links recorded actions by admitted cause edges; it does not
  prove observation honesty or completeness.
- A traced action is not proof that the action succeeded.
