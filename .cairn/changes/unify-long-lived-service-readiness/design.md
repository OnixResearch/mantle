# Design: One readiness vocabulary for long-lived Mantle services

## Goal and scope

Give long-lived components and declared proof gates one readiness model with
explicit dependencies and restart policy. Keep the scope to three named
consumers.

## Current behavior

The Rust cache daemon serves requests after a policy load and a peer check. A
remote serve binding accepts a session after binding setup. Proof stages run in
declared order. Each site expresses readiness its own way, and no shared report
answers which components are ready.

## Approach review

| Family | Mechanism | Disposition | Required check |
| --- | --- | --- | --- |
| Keep per-component readiness | Document each handshake | Rejected: no shared vocabulary, dependencies stay implicit | Dependency fixture |
| Adopt the reviewed vocabulary | `started`/`ready`/`complete`/`failed` plus declared states | Selected direction | Readiness and dependency fixtures |
| General supervisor in Mantle | Own process supervision for user programs | Rejected: out of scope for a build tool (ADR 0010) | Not blocking |
| Readiness in the coordination surface only | Publish facts, keep no local states | Rejected: the daemon needs local decisions too | Daemon fixture |

## Contract and component ownership

- Pure core: state vocabulary validation, dependency graph evaluation, restart
  policy normalization, and derived-state computation over in-memory facts.
- Shell: component startup, signal handling, binding setup, policy load, and
  doctor rendering.
- Surface: derived readiness state publishes on the coordination surface when
  it exists, and stays optional otherwise.

## Decisions

### Decision: `ready` is asserted in addition to `started`

**Choice:** A component reports `started` when its process or binding exists
and `ready` only when it can take work.

**Rationale:** The distinction is the whole point of a readiness model. The
reviewed manual keeps `ready` as an additional assertion for the same reason.

### Decision: Dependencies are declared, not inferred

**Choice:** Each component declares what it depends on. The evaluator does not
infer dependencies from call order.

**Rationale:** Inferred order is what the code already does. Explicit
declarations make a blocked component visible instead of silent.

### Decision: Restart policy is a closed matrix

**Choice:** Every supervised component names one of `always`, `on-error`,
`all`, `never`.

**Rationale:** A closed set is reviewable. The reviewed matrix already covers
the cases Mantle needs: restart on failure, restart always, treat all exits as
normal, or restart the whole daemon.

## Risks / Trade-offs

- New states can drift from real behavior. The fixtures check readiness against
  an actual request, not a flag.
- Doctor output grows. New fields are additive and versioned.
- A blocked dependency must not stall silently; the report names the blocking
  component.

## Non-Claims

- Declared readiness proves the component's own observation.
- A `ready` component is not proof of output quality, correctness, or release
  eligibility.
