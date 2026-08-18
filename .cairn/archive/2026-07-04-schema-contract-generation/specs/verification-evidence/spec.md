## ADDED Requirements

### Requirement: Machine JSON schemas have generated Nickel contract evidence [r[verification_evidence.machine_schema_contracts]]

Mantle MUST maintain a deterministic schema-contract rail for selected machine-readable JSON surfaces. The rail MUST bind each selected surface to a schema owner, generated or checked Nickel contract, freshness check, positive fixtures, negative fixtures, unsupported-schema diagnostics, and bounded non-claims.

#### Scenario: Valid fixture satisfies generated contract [r[verification_evidence.machine_schema_contracts.scenario.valid]]

- GIVEN a selected Mantle JSON surface has a schema owner and generated Nickel contract
- WHEN the schema-contract rail validates a positive fixture for that surface
- THEN the fixture MUST satisfy the generated contract
- AND the evidence MUST identify the surface, schema identity, contract identity, and fixture identity.

#### Scenario: Invalid fixture is rejected [r[verification_evidence.machine_schema_contracts.scenario.invalid]]

- GIVEN a selected Mantle JSON surface has negative fixtures for malformed or unsupported payloads
- WHEN the schema-contract rail validates those fixtures
- THEN each invalid fixture MUST fail with a deterministic validation outcome
- AND the outcome MUST distinguish contract rejection from unsupported-schema or fixture setup failures.

#### Scenario: Stale generated contract fails freshness check [r[verification_evidence.machine_schema_contracts.scenario.stale]]

- GIVEN a JSON schema or schema owner changes without refreshing the generated Nickel contract or snapshot
- WHEN the schema-contract rail runs
- THEN Mantle MUST report a stale-generated-contract diagnostic
- AND it MUST NOT claim that the affected JSON surface contract is current.

#### Scenario: Schema validation claim is bounded [r[verification_evidence.machine_schema_contracts.scenario.non-claim]]

- GIVEN the schema-contract rail passes for selected fixtures
- WHEN Mantle reports evidence, task completion, release readiness, or docs status
- THEN the claim MAY state that selected fixtures satisfy or fail the selected schema contracts
- AND it MUST NOT claim command correctness, build correctness, release reproducibility, deployability, or frontend module correctness without separate current evidence.
