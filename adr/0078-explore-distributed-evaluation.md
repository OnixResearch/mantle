# ADR 0078: Explore distributed evaluation feasibility

## Status

Accepted

## Context

Mantle distributes realization but not evaluation. Remote workers must receive
concrete build inputs that the client already evaluated or lowered. They never
evaluate Nickel source.

The question of whether evaluation itself can be distributed was open. Several
seams exist: `IsolatedWorkerInput` worker sessions, the ADR 0074 strict worker
protocol, evaluation-stream workers, evaluation budget framing, the portable
client boundary, and content-addressed source staging.

A bounded assessment was needed before any authority change. The exploration
recorded the seam inventory, ran a real eval-worker round-trip probe over a
framed stdio transport, and classified two candidate routes.

## Decision Drivers

- Keep evaluation, remote-build, scheduler, provider, and publication
  authority unchanged during the assessment.
- Test the worker boundary with a real process round trip, not a design
  narrative.
- Classify each route deterministically from bound evidence.
- Preserve the concrete-input contract that remote workers already accept.

## Decision

The assessment outcome is recorded in the change report. The worker boundary
is transportable for the probed fixture: a fresh evaluation session built from
declared request facts reproduced the client result byte-for-byte.

The two candidate routes classify as follows:

- Evaluate-once producer: `candidate`. The shape already exists through
  `nickel-export-core` and `mantlepkgs`. Distribution in time is compatible
  with the current design.
- Eval-as-a-service: `blocked`. The probe passed, but cross-boundary evidence
  is missing for streaming evaluation-to-build overlap, dynamic-goal
  admission, and evaluator suspension.

No authority changed. A `candidate` outcome authorizes only a later Cairn
change with separate product evidence.

## Alternatives Considered

### Treat the passed probe as adoption evidence

Rejected. A single-fixture round trip proves the seam is transportable. It
does not prove streaming or dynamic-goal behavior survives a network
transport.

### Treat the authority boundary as a hard blocker

Rejected. The client-evaluates boundary is policy (ADR 0010), not code. A
later accepted change can relax it without breaking the concrete-input
contract.

### Classify eval-as-a-service as rejected

Rejected. `blocked` is accurate: the blocker is missing evidence, not a
demonstrated defect.

## Consequences

- The worker boundary seam is now evidenced as transportable for a
  single-root fixture.
- Streaming overlap, dynamic-goal admission, and evaluator suspension are the
  named preconditions for an eval-service route.
- The evaluate-once route matches the industry consensus (REAPI, Hydra) and
  the existing producer pattern.
- Future work must demonstrate cross-boundary streaming, dynamic goals, and
  suspension before an eval-service route opens.
- The assessment does not prove evaluator equivalence, remote-build
  correctness, or release eligibility.
