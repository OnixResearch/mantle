# Design: Codify transient-handle admission

## Goal and scope

Name the transient-handle rule, apply it to each existing protocol boundary,
and add the missing negative fixtures. No wire change is planned.

## Current behavior

Transfer chunks are attributed to a canonical manifest that binds the session,
job, attempt, fence generation, policy digest, store prefix, and artifact set.
A checkpoint cursor is explicitly not proof of content. An attempt is installed
under a lease before dispatch. Output admission requires signed PathInfo,
content, requested output, prefix, and attestation facts. These are all
instances of the same rule, and the rule is not written down in one place.

## Approach review

| Family | Mechanism | Disposition | Required check |
| --- | --- | --- | --- |
| Leave the rule implicit | Rely on per-boundary code and docs | Rejected: new boundaries can miss it | Boundary inventory fixture |
| State the rule, keep behavior | Documentation plus one negative fixture per boundary | Selected direction | Per-boundary negative fixtures |
| Add a protocol-level rejection message | Feedback on unknown handles | Rejected: the reviewed model gives no protocol feedback | Not blocking |
| Central handle registry | One service validates every handle | Deferred: larger change than the rule statement needs | Not blocking |

## Contract and component ownership

- Pure core: declaration and handle-shape validation, introduction checks, and
  the per-boundary rule table over in-memory facts.
- Shell: the existing boundary code that consumes the decision.
- Documentation: `docs/remote-transfer.md` names the rule per boundary. An ADR
  records the decision.

## Decisions

### Decision: Rule statement plus fixtures, not a redesign

**Choice:** Keep existing behavior. Add validation coverage where a boundary
lacks it.

**Rationale:** The protocol already behaves correctly in the reviewed paths.
The gap is reviewability and regression coverage.

### Decision: No protocol-level feedback

**Choice:** An unknown handle fails closed without a rejection message.

**Rationale:** The reviewed model explains why: a live peer cannot distinguish
rejection from death. Mantle's bounded transfer already fails closed quietly.

### Decision: One boundary table

**Choice:** Keep a single table that names, for each boundary, the declaration
that introduces the handle and the fixture that rejects an unknown handle.

**Rationale:** A table makes the next protocol field reviewable in one place.

## Risks / Trade-offs

- A boundary may be found to accept an unknown handle. That becomes a fix in
  this change, with the fixture proving it.
- Documentation can drift. The fixture list is the check.

## Non-Claims

- The rule proves protocol shape only.
- It does not prove delivery, completeness, crash consistency, or content
  trust.
