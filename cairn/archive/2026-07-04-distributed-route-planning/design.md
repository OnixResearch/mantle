## Context

The accepted realization-routing model already names remote-builder capabilities, source readiness, substituter trust, archives, local executor facts, network policy, and evidence strength as planner inputs. The current build CLI still treats explicit remote dispatch as incompatible with `--plan`, so remote viability is discovered too late.

## Decisions

### 1. Remote route planning is pure

**Choice:** The route core consumes explicit remote-builder facts supplied by the shell and never opens sockets, redeems tickets, scans remote stores, reads hidden config, or uploads inputs.

**Rationale:** Operators need replayable plans and deterministic rejected-route reason codes before mutation.

### 2. Route reports separate eligibility from execution

**Choice:** A remote-eligible plan says only that the supplied facts make a remote route selectable. It does not claim that a remote session was opened, inputs were uploaded, outputs were built, or trust was admitted.

**Rationale:** This preserves proof-before-claim and keeps `--plan` safe.

### 3. Ranking is fact-driven

**Choice:** Remote routes are ranked using configured policy facts and stable tie-breakers, not discovery order or first response latency.

**Rationale:** Equivalent plans should produce equivalent reports on every host.

## Risks / Trade-offs

- Remote fact collection must stay bounded and redacted or plan output can become a secret leak.
- Strong evidence mode may reject routes that are usable for practical builds but lack receipt material.
