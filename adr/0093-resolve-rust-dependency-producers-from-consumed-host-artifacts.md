# ADR 0093: Resolve Rust dependency producers from consumed host artifacts

## Status

Accepted (2026-08-31)

## Context

V90 passed source-identity framing and reached the next Rust action-planning
boundary.

The `strum_macros` proc-macro unit had a `rustversion` dependency artifact with
no direct `producer_unit_id`. The same unit also carried a consumed host
artifact for the exact `rustversion` proc-macro producer.

Native Rust planning uses this shape when a host proc-macro edge repairs a
Cargo artifact that omitted its host producer. The action adapter rejected the
directly unbound field without consulting the already selected host artifact.

## Decision Drivers

- Use only producer facts already present in the unit plan.
- Prefer an explicit dependency producer when present.
- Support native proc-macro host-edge repair.
- Reject missing and ambiguous fallback producers.
- Bind the selected producer in both action ordering and input authority.
- Do not infer producers from ambient package discovery.

## Decision

For each dependency artifact:

1. Use its direct `producer_unit_id` when present.
2. Otherwise, select consumed host artifacts with the same package identity.
3. Prefer candidates whose target name equals the dependency artifact name.
4. If no exact-name candidate exists, permit only one package-level candidate.
5. Require exactly one distinct producer identity.
6. Reject zero or multiple producer identities.

Add the selected identity to the unit's producer set. Also include it in the
canonical dependency input authority string.

Consumed host artifacts and build metadata still require their own explicit
producer identities. This decision does not relax those checks.

## Alternatives Considered

### Reject every missing direct producer

Rejected. Native planning already supplies a stronger selected host-artifact
fact for this bounded proc-macro edge.

### Use the first matching host artifact

Rejected. Iteration order is not authority, and duplicate producers must fail.

### Match only by package identity

Rejected as the first choice. Exact target-name matches prevent wrong sibling
host targets from winning.

### Search all graph units for a matching package

Rejected. The unit's consumed host artifacts are its declared capability set.

## Consequences

- Proc-macro units can bind Cargo-omitted host producer edges.
- Renamed dependencies can use a unique package-level fallback.
- Missing or ambiguous producer facts still fail before execution.
- Action ordering and input authority use the same resolved producer.
- V90 remains failed evidence. A fresh promoted proof must verify this rule.
