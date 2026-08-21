# Design: versioned Mantle build interchange

## Context

Mantle owns derivation evaluation, scheduling, sandbox execution, store and cache behavior, and build evidence. Kiln owns CI attempts, candidate identity, required product admission, and outcome policy.

The aggregate Mantle build report does not name Kiln facts. Adding consumer-specific facts to the native report would mix authority. The new contract wraps a bounded Mantle observation with an exact consumer request identity.

## Goal and Success Contract

Publish one host-independent component that binds a Mantle build observation to an exact external request.

Completion requires versioned Rust and Nickel schemas, deterministic identities, exact linkage, product admission, coherent outcomes, producer fixtures, and positive and negative tests.

False completion includes importing Mantle runtime crates into a consumer core, treating cache use as success, accepting missing products, trusting supplied identities without recomputation, or claiming build correctness.

## Decisions

### Decision: Publish a standalone functional core

**Choice:** Add `mantle-build-contract` as a `no_std + alloc` crate with pure values, framing, validation, and admission.

**Rationale:** Consumers can pin one immutable revision without importing the Mantle CLI, store, scheduler, or sandbox.

### Decision: Bind consumer applicability outside the native report

**Choice:** The request binds effect, attempt, candidate, pipeline, plan, policy, idempotency, platform, and sorted required outputs. The observation repeats the request identity and binds outcome, products, builder, worker, store, cache, logs, metrics, and receipt identities.

**Rationale:** Mantle keeps native build meaning. Consumers keep candidate and CI meaning.

### Decision: Admit products before success

**Choice:** Success requires every sorted unique required product exactly once. Cache and substitution facts remain advisory fields. Missing or duplicate products reject success.

**Rationale:** A cache hit or provider success cannot replace consumer-owned product admission.

### Decision: Keep execution and recovery in consumer shells

**Choice:** The contract performs no process, filesystem, transport, clock, credential, cancellation, persistence, or retry behavior.

**Rationale:** Those effects and policies belong to the consumer shell. Missing or ambiguous output does not fabricate an observation.

## Risks / Trade-offs

- Version one binds a normalized aggregate observation, not every internal Mantle build event.
- Consumer-supplied candidate and plan facts are bound but not authenticated by Mantle.
- The contract does not prove build correctness, sandbox completeness, cache truth, product semantics, reproducibility, or release readiness.
