# Design: Separate distributed core and shell

## Context

The distributed build module contains deterministic key and route logic. It also contains async service traits, concrete in-memory adapters, remote and local build-service adapters, and Snix/store values.

This shape prevents a direct infrastructure-free test boundary. It also lets adapter return types become Mantle decision inputs without one explicit translation boundary.

The external batch port has similar coupling. Its contract is declared beside concrete process adapters and uses raw text for all failures.

## Goal and Success Contract

The distributed core must own build meaning. The application shell must own service calls, ordering, retries, cancellation, time, and effect execution.

Completion requires these results:

- pure decisions use only Mantle-owned domain values;
- Snix, store, Slurm, process, and async runtime types remain outside the pure decision boundary;
- application-owned ports use typed requests, observations, and failures;
- adapters translate infrastructure values without assigning domain meaning;
- visible composition roots select adapter order and concrete implementations; and
- positive and negative tests cover each boundary.

False completion includes moving the current module, renaming traits, or wrapping vendor values in type aliases.

## Decisions

### Decision: Split facts and decisions from service execution

A pure distributed core owns realization keys, route eligibility, deterministic ranking, accepted artifact facts, outcome admission, fallback policy, and effect plans.

The core receives all current store, capability, trust, network, time, and prior-operation facts explicitly. It never awaits, calls a port, or constructs a concrete service.

### Decision: Project infrastructure values into Mantle values

Adapters convert Snix `PathInfo`, store substitution reports, build-service responses, and provider data into Mantle-owned bounded observations.

The projection preserves every protocol-required field and native identity. It does not treat a vendor object as trusted merely because decoding succeeded.

### Decision: Put ports with the application shell

The application layer owns narrow capabilities for artifact resolution, artifact publication, local or remote realization, and external batch operations.

These ports use Mantle requests, observations, and typed application failures. Concrete store, Snix, direct-process, and Slurm modules implement them.

### Decision: Keep orchestration in the shell

The shell calls resolvers in the planned order, executes only selected effects, obtains time, applies cancellation, and records observations.

After each call, the shell passes bounded observations back to the core. The core decides reuse, fallback, unknown state, retry, or terminal disposition.

### Decision: Separate error categories

Domain errors describe invalid facts, ineligible routes, stale fences, and rejected transitions.

Infrastructure errors describe serialization, unavailable services, process failure, timeout, cancellation, malformed output, and storage failure. The shell translates them without erasing ambiguity.

### Decision: Preserve one visible composition root

One composition root selects resolver order, local and remote realizers, store adapters, and external batch adapters from validated configuration.

No core or adapter silently constructs a fallback provider. A new selection requires a new explicit decision input.

## Compatibility

Keep accepted PathInfo semantics, external batch protocol bytes, route results, BLAKE3 identities, and supported public reports stable.

Use explicit bidirectional projections and temporary compatibility adapters. Remove them only after all maintained callers and golden fixtures pass.

## Validation

Pure tests cover deterministic route choice, rejection, fallback denial, stale observations, unknown outcomes, and accepted artifact facts.

Shell tests cover call order, no effect after denial, retry limits, cancellation, timeout, transaction boundaries, and no silent fallback.

Adapter tests cover Snix/store projection, direct-process and Slurm translation, malformed output, exceeded bounds, process failure, and unavailable services.

Dependency and source-shape checks reject vendor types, async traits, concrete adapters, and effect calls from the declared pure distributed boundary.

## Claim Boundary

Passing checks proves only the tested split, decision behavior, and translation behavior. It does not prove remote execution, scheduler behavior, cache integrity, or output trust.
