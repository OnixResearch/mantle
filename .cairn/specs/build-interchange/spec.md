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

### Requirement: Library hash dependency does not inherit store trait coupling
r[mantle.build_interchange.hash_dependency] The standalone contract MUST permit compatible BLAKE3 1.x selection without changing store digest-trait pins, default-feature policy, wire identity framing, or admission semantics.

#### Scenario: Consumer requires a newer compatible hash
- GIVEN the contract uses only inherent BLAKE3 hashing APIs and the consumer requires 1.8.7
- WHEN Cargo resolves the standalone contract
- THEN the consumer MUST resolve without forcing an older authority dependency or a local version override

#### Scenario: Owner workspace retains its store dependency
- GIVEN the owner workspace locks BLAKE3 1.8.2 for its digest-trait boundary
- WHEN the standalone contract's compatible range changes
- THEN the owner MUST retain that lock and its store pins without changing the selected native hash implementation

### Requirement: Exact hash compatibility matrix
r[mantle.build_interchange.hash_matrix] Conformance MUST test independently locked BLAKE3 1.8.2 and 1.8.7 consumers against unchanged producer fixtures, rejection controls, and host-independent compilation.

#### Scenario: Original identities remain stable
- GIVEN the retained original request, direct success, cache success, and failure fixtures
- WHEN either matrix consumer recomputes and validates the values
- THEN the original identities MUST remain valid without fixture regeneration

#### Scenario: Recomputed malformed observation
- GIVEN a changed builder, missing required product, absent cache source, or contradictory unknown receipt with a recomputed identity
- WHEN either consumer runs admission
- THEN the existing deterministic rejection MUST remain effective

### Requirement: Published linked consumer proof
r[mantle.build_interchange.linked_consumer] Compatibility evidence MUST identify a published immutable source and an actual linked consumer cohort, and MUST NOT treat dependency resolution as native execution or promotion authority.

#### Scenario: Product graph links the published fix
- GIVEN Neural Stream retains its admitted Animus revision and BLAKE3 1.8.7
- WHEN it checks the published contract source without local overrides
- THEN the linked checks MUST pass before that contract scope is admitted

#### Scenario: Package compatibility is not training evidence
- GIVEN a successful linked contract check
- WHEN evidence is recorded
- THEN it MUST disclaim native CLI revalidation, output authenticity, trainer execution, reproducibility, and production promotion
