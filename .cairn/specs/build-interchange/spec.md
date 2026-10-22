# Build Interchange Specification

## Purpose

Defines the `build-interchange` capability.

## Requirements

### Requirement: Versioned build interchange contract
r[mantle.build_interchange.contract] Mantle MUST publish a versioned host-independent build interchange contract with bounded request, observation, product, metric, log, cache, builder, worker, store, and receipt values.

#### Scenario: Consumer pins contract
r[mantle.build_interchange.contract.scenario.pinned]
- GIVEN a consumer pinned to one immutable Mantle revision
- WHEN it decodes a conforming request or observation
- THEN the contract MUST expose the typed value without Mantle runtime or host effects.

#### Scenario: Schema differs
r[mantle.build_interchange.contract.scenario.schema]
- GIVEN a value with an unsupported schema
- WHEN admission runs
- THEN Mantle MUST reject it before any external effect.

### Requirement: Exact build request identity
r[mantle.build_interchange.request] Each request MUST bind effect, attempt, candidate, pipeline, plan, policy, idempotency, platform, and sorted unique required-output identities through deterministic BLAKE3 framing.

#### Scenario: Complete request
r[mantle.build_interchange.request.scenario.valid]
- GIVEN a bounded request with canonical required outputs
- WHEN request admission runs
- THEN Mantle MUST recompute and accept its exact identity.

#### Scenario: Candidate changes
r[mantle.build_interchange.request.scenario.substitution]
- GIVEN a request whose candidate changes without a new identity
- WHEN admission runs
- THEN Mantle MUST reject the request.

### Requirement: Exact build observation admission
r[mantle.build_interchange.observation] Each observation MUST bind the exact request, outcome, products, builder, worker, store, cache, log, metric, and receipt identities.

#### Scenario: Linked success
r[mantle.build_interchange.observation.scenario.success]
- GIVEN a successful observation linked to an admitted request
- WHEN observation admission runs
- THEN Mantle MUST admit it only after product checks pass.

#### Scenario: Builder differs
r[mantle.build_interchange.observation.scenario.builder]
- GIVEN an observation whose builder differs from the admitted cohort
- WHEN observation admission runs
- THEN Mantle MUST reject it.

### Requirement: Required product admission
r[mantle.build_interchange.products] A successful observation MUST contain every sorted unique required product exactly once, and cache facts MUST NOT bypass this check.

#### Scenario: Cache hit misses product
r[mantle.build_interchange.products.scenario.cache]
- GIVEN a cache-backed success without one required product
- WHEN product admission runs
- THEN Mantle MUST reject success.

#### Scenario: Products match
r[mantle.build_interchange.products.scenario.match]
- GIVEN a success with every required product exactly once
- WHEN product admission runs
- THEN Mantle MAY admit the product set.

### Requirement: Build interchange conformance
r[mantle.build_interchange.conformance] Maintained conformance MUST include deterministic producer fixtures, positive and negative Rust tests, typed Nickel fixtures, WebAssembly compilation, and fixture freshness checks.

#### Scenario: Fixture drifts
r[mantle.build_interchange.conformance.scenario.drift]
- GIVEN a checked-in producer fixture that differs from current contract output
- WHEN conformance checks run
- THEN the checks MUST fail.

### Requirement: Build interchange boundary
r[mantle.build_interchange.boundary] Build interchange documentation and observations MUST preserve Mantle build authority, consumer CI authority, exact applicability, and explicit non-claims.

#### Scenario: Success admitted
r[mantle.build_interchange.boundary.scenario.scope]
- GIVEN an admitted successful observation
- WHEN a consumer records it
- THEN it MUST NOT claim build correctness, sandbox completeness, cache truth, product semantics, reproducibility, or release readiness.

### Requirement: The building plane emits bounded live facts

r[mantle.build_interchange.live_state_emission] The building plane MUST emit
bounded, versioned facts and events for goal state, worker presence,
reservations, and terminal outcomes. Fact identity MUST come from the building
plane.

Emission MUST NOT block, MUST NOT change build behavior, and MUST NOT be a
build precondition. A missing or failed coordination endpoint MUST produce a
degraded observation event and MUST NOT fail the build.

The live daemon owner id MUST identify the individual emitting invocation.
The content-addressed evaluation-stream run id MUST remain unchanged and MUST
NOT be reused as live owner identity for concurrent identical inputs.

#### Scenario: Build without a coordination daemon

- GIVEN a build with live-state emission enabled and no daemon reachable
- WHEN the build runs to completion
- THEN it MUST produce the same outputs and receipts as a build with emission
  disabled
- AND it MUST report degraded observation

#### Scenario: Concurrent identical-input builds remain independent

- GIVEN two simultaneous builds with identical source and selected roots
- WHEN both opt into the same live daemon
- THEN each MUST publish under a different owner id without duplicate-owner
  degradation
- AND stopping either build MUST NOT retract the other build's facts
- AND their NDJSON/report identities, outputs, and receipts MUST remain unchanged

#### Scenario: Emission does not block dispatch

- GIVEN a slow or stalled coordination endpoint
- WHEN the scheduler dispatches goals
- THEN emission MUST NOT delay dispatch beyond the declared emission bound
