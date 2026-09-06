# Build Interchange Hash Compatibility

## ADDED Requirements

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
