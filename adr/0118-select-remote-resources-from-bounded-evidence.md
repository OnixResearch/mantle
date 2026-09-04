# ADR 0118: Select remote resources from bounded evidence

## Status

Accepted

## Context

Mantle already enforces declared resource minima and owns fenced worker leases.
Historical observations can improve machine-class selection. They can also add
unsafe authority if they are stale, opaque, unbounded, or allowed to weaken a
request.

OOM retries, quota accounting, and cross-project result sharing have separate
authority boundaries. One large scheduler module would mix these boundaries
with worker execution and store access.

Valence already defines the accepted build-service evidence linkage profile.
OnixOS owns machine declarations. ChaosControl owns fault execution.

## Decision

Mantle adds `crunch-resource-policy-core` as a `no_std + alloc` functional core.
It accepts only explicit facts. It owns action-family identity, observation
admission, deterministic selection, OOM escalation, usage planning, sharing,
benchmark comparison, fault expectations, rollout controls, and non-claims.

`crunch-resource-policy` owns the application ports for usage-ledger commits
and Valence evidence publication. Root adapters map decisions to existing
coordinator, fenced-attempt, lease, and strong action-result authorities.

Declared CPU, memory, scratch, architecture, platform, KVM, trust, isolation,
and feature requirements remain hard lower bounds. Historical facts can only
retain or increase those requirements.

A retry requires trusted platform-specific OOM evidence. It also requires a
strictly larger eligible class and must remain inside retry, ordinal, charge,
wall-time, and quota limits. The existing attempt core creates the new attempt
and advances its fence.

Usage reservation and reconciliation use compare-and-commit semantics.
Duplicate identical records are idempotent. Conflicts fail closed. A failed
store commit returns no quota grant.

Result sharing remains private by default. Explicit sharing still depends on
the existing strong action-result decision, signature, policy, platform,
output, authorization, and CAS facts.

All authority features default to off in observe-only mode. Historical
selection, OOM retry, quota enforcement, and result sharing have independent
controls.

## Consequences

- The same canonical inputs and policy version produce the same decision.
- Scheduler integration reserves the effective historical minima, not only the
  original static quantities.
- Valence validates linkage and roles but does not own Mantle policy.
- OnixOS source identity does not prove current worker capacity or trust.
- ChaosControl fixtures describe bounded expected fault outcomes. They do not
  prove arbitrary production behavior.
- The benchmark fixture adapts only the reviewed workload shape from
  `nixbuild/nixbench`; it does not copy source or hosted-service policy.

## Non-claims

Resource-policy evidence does not prove future sufficiency, build correctness,
host isolation, fair billing, production performance, or release eligibility.
Shared CAS presence does not authorize reuse. A successful benchmark does not
promote an observation into a guarantee.

## Rejected alternatives

### Add policy to the existing lease module

Rejected. Lease capacity and historical policy have different owners and
change rates.

### Let an opaque service select classes

Rejected. An opaque selector cannot provide deterministic replay or stable
reason codes.

### Retry all failed builds on a larger machine

Rejected. Arbitrary retries waste quota and convert failure ambiguity into
execution authority.
