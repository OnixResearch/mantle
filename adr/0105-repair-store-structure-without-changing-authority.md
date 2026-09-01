# ADR 0105: Repair Store Structure Without Changing Authority

**Status:** Accepted

## Context

The strict Tiger Style gate reported 183 focused findings in `crunch-store`.
The findings covered long functions, weak assertion density, implicit bounds,
ambiguous interfaces, host-sized public indices, and hidden arithmetic risks.

A mechanical lint cleanup could change store meaning. Assertions can replace
typed input errors with panics. Helper extraction can reorder validation and
mutation. Broad interfaces can also blur functional-core and shell authority.

## Decision

Keep malformed external input on typed rejection paths. Use assertions only for
facts established by types, prior validation, checked bounds, or successful
observations.

Split long functions at existing observation, validation, planning, and effect
boundaries. Preserve effect order. Use explicit worklists for bounded recursive
shape processing.

Use named request records for ambiguous private interfaces. Use `u32` for the
public store-layer index and perform checked conversions at host collection
boundaries.

Do not add Tiger allowances, warning budgets, finding baselines, or narrower
check scope.

## Consequences

- Store lifecycle cores retain deterministic policy ownership.
- The store shell retains filesystem, service, network, signing, publication,
  and mutation authority.
- Layer identity is stable across host pointer widths.
- Internal impossible states are visible near the logic they protect.
- Positive and negative tests must pass before and after structural changes.
- Later Tiger findings remain separate blockers and cannot inherit acceptance
  from this repair.
