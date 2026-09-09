# Design: Extract the build-planning core

## Context

Mantle already has a pure realization-routing kernel. Shell modules still collect facts and make some final cache, substitute, build, preflight, and concurrency decisions.

The target flow is:

```text
build request
  -> shell observes store, source, doctor, remote, platform, and policy facts
  -> build-planning core
  -> selected route, rejected reasons, job limit, blockers, and effect plan
  -> shell executes the plan
  -> observations become execution evidence
```

## Decisions

### Decision: separate observations from route decisions

The shell will gather bounded facts through narrow capabilities. The core will receive one admitted `BuildPlanningFacts` value and return a deterministic `BuildPlanningDecision`.

The core will not read paths, stores, environment state, clocks, remote services, keys, or doctor output.

**Rationale:** Equivalent facts must produce the same plan regardless of observation order or host timing.

### Decision: make concurrency inputs explicit

Concurrency policy will receive requested jobs, observed available parallelism, the configured policy cap, and any explicit executor limit. It will use checked conversion and return a typed bounded job count or blocker.

The shell alone can call `available_parallelism` or inspect provider capacity.

**Rationale:** Host observation is nondeterministic input. The clamp policy is deterministic domain logic.

### Decision: keep route selection distinct from execution

The decision will name the selected route, ordered rejected routes, stable reason codes, requested claim strength, and a bounded effect plan. It will not open a remote session, redeem credentials, query a substituter, mutate a store, run a build, or publish an output.

The shell will recheck plan-bound observations before a mutating effect where accepted policy requires freshness.

### Decision: use typed planning blockers

Store absence, source gaps, output-trust gaps, unsupported platform, offline network need, and executor absence will remain separate blocker variants. Presentation adapters will map them to existing human and JSON diagnostics.

A generic shell `RunError` will not enter the planning core.

### Decision: preserve current policy through golden matrices

A checked fixture matrix will cover local cache, substitution, source bundle, remote builder, local build, and preflight rejection. Legacy and extracted paths must select the same route, reason order, and job limit before cutover.

## Ports and adapters

The application shell will consume capabilities for local output observation, source readiness, substituter and archive candidates, remote candidates, doctor facts, platform facts, and host parallelism. These are observation capabilities, not policy ports.

## Testing and evidence

Tests will cover every route, multiple simultaneous blockers, offline mode, strong claim requirements, missing facts, zero and excessive parallelism, deterministic ordering, host-state poisoning, stale observations, and adapter failures.

Evidence proves the selected plan under supplied facts. It does not prove that an effect ran, a remote worker succeeded, an output was admitted, or the route was optimal.

## Risks

- Observation DTOs can become unbounded snapshots. Every collection and string field needs a named limit.
- Route execution can silently re-plan. The shell must execute the accepted plan or return a drift blocker.
- Golden compatibility can hide non-determinism. Fixtures must vary insertion and discovery order.
