# Machine Artifact Contract Specification

## ADDED Requirements

### Requirement: Public machine artifacts are completely classified

r[mantle.machine_artifact_contracts.inventory] Mantle MUST maintain a Nickel inventory that classifies every public machine-readable JSON surface as contracted, internal, debug, compatibility, or external and records its Rust owner, producer, consumers, artifacts, version policy, validation commands, fixture coverage, freshness strategy, and non-claims.

#### Scenario: Public JSON has no registry decision

- GIVEN a CLI command or public API emits machine JSON not present in the inventory
- WHEN machine-contract validation runs
- THEN validation MUST fail and identify the unclassified producer before release.

### Requirement: Runtime DTO and review contract authority is one-way

r[mantle.machine_artifact_contracts.authority] For contracted runtime-emitted artifacts, Rust DTOs MUST own emitted facts, an exact schema snapshot MUST be deterministically derived or rendered from that owner, and the Nickel contract MUST be generated from the supported schema subset with serialization parity evidence.

#### Scenario: Schema or generated contract drifts from Rust

- GIVEN the Rust serialization shape changes without a matching schema and contract update, or the schema uses an unsupported construct
- WHEN generation/parity validation runs
- THEN validation MUST fail rather than retain a stale contract or degrade the unsupported field to a permissive type.

### Requirement: The registry drives contract checking

r[mantle.machine_artifact_contracts.registry_rail] The machine-contract checker MUST iterate contracted inventory entries and validate each entry's owner, schema, generated Nickel contract, fixtures, consumer/version policy, and non-claims without report-family-specific control flow in the imperative shell.

#### Scenario: A new contracted surface is registered

- GIVEN a complete registry entry and supported schema are added
- WHEN the checker runs
- THEN the generic rail MUST discover and validate the surface
- AND adding the surface MUST NOT require a new hard-coded checker branch.

### Requirement: Generated contracts use semantic scalar vocabulary

r[mantle.machine_artifact_contracts.contract_vocabulary] Generated Nickel contracts MUST use shared exact schema, closed enum, bounded integer, lowercase BLAKE3, safe logical reference, bounded collection, and redaction-safe text contracts where those invariants apply.

#### Scenario: Generated record falls back to broad dynamic data

- GIVEN a contracted field has a declared enum, digest, bound, reference, or redaction invariant
- WHEN its Nickel contract is generated
- THEN the generated contract MUST preserve that invariant
- AND it MUST NOT replace the field with `Dyn` or an unconstrained scalar.

### Requirement: The initial cohort covers high-impact boundaries

r[mantle.machine_artifact_contracts.initial_cohort] Mantle MUST contract the selected stable build report/plan, realization-route plan, portable-receipt bundle/verify/import, source-bundle plan/verify/offline-preflight, Nickel export report/receipt, and release/attestation handoff projections before claiming registry coverage for the initial cohort.

#### Scenario: Initial cohort is reported complete

- GIVEN registry validation reports the initial cohort complete
- WHEN the inventory is inspected
- THEN every named family MUST have an owner, schema, generated contract, positive fixtures, negative fixtures, version policy, and consumer non-claim.

### Requirement: Machine contract fixtures cover success and failure

r[mantle.machine_artifact_contracts.fixtures] Every contracted surface MUST include Rust-serialized positive fixtures and negative fixtures for schema/version, vocabulary, digest, reference, bounds, unknown-field, redaction, and applicable cross-field failures.

#### Scenario: Invalid artifact reaches a consumer

- GIVEN a fixture omits its schema, uses an unsupported version or enum, carries a malformed digest or unsafe reference, exceeds a declared bound, leaks a forbidden field, or violates a family invariant
- WHEN contract validation runs
- THEN the fixture MUST be rejected with a deterministic surface and issue class.

### Requirement: Compatibility versions are explicit

r[mantle.machine_artifact_contracts.versioning] Contracted consumers MUST reject unsupported schema versions by default, and any admitted prior version MUST have an explicit compatibility classification, deterministic converter, and positive and negative migration fixtures.

#### Scenario: Unknown future version is supplied

- GIVEN a consumer receives a schema version absent from its inventory entry
- WHEN it parses the artifact
- THEN it MUST fail closed without treating the artifact as the current version.

### Requirement: Schema, contract, and fixture freshness is content-bound

r[mantle.machine_artifact_contracts.freshness] The registry rail MUST bind each contracted surface's schema, generated Nickel contract, fixture set, producer identity, and consumer/version policy with BLAKE3 and MUST fail when any bound artifact changes without regeneration and review.

#### Scenario: Contract snapshot is stale

- GIVEN a schema or producer shape changes while the generated contract and fixtures remain unchanged
- WHEN freshness validation runs
- THEN validation MUST fail before packaging and identify the stale bound artifact.

### Requirement: Nickel contracts do not produce runtime reports

r[mantle.machine_artifact_contracts.runtime_boundary] Mantle MUST serialize and validate runtime build, cache/reuse, import, receipt, attestation, and release artifacts in Rust; Nickel evaluation MUST occur only in explicit generation/check workflows.

#### Scenario: Contracted artifact passes review validation

- GIVEN an artifact satisfies its schema and Nickel contract
- WHEN its result is summarized
- THEN the summary MUST NOT claim build correctness, cache trust, reproducibility, release eligibility, attestation truth, or deployability solely from contract conformance.
