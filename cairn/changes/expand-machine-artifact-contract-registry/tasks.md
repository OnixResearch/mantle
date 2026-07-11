## Phase 1: Inventory and authority

- [ ] [serial] Discover every public machine JSON producer and classify it as contracted, internal, debug, compatibility, or external with Rust owner, command/API, consumers, artifacts, version policy, validation, and non-claims. r[mantle.machine_artifact_contracts.inventory]
- [ ] [serial] Harden `schemas/machine-contracts/inventory.ncl` with exact entry classes, required fields, unique surface ids and artifact paths, complete fixture declarations, and BLAKE3 identities. r[mantle.machine_artifact_contracts.inventory]
- [ ] [serial] Define the Rust DTO to schema snapshot to generated Nickel contract authority flow and reject unsupported schema constructs instead of emitting permissive contracts. r[mantle.machine_artifact_contracts.authority]

## Phase 2: Data-driven contract rail

- [ ] [serial] Extract pure registry, schema-subset, contract-rendering, fixture-classification, and freshness cores from the doctor-specific checker, leaving file/process handling in a thin shell. r[mantle.machine_artifact_contracts.registry_rail]
- [ ] [serial] Add shared generated-contract primitives for exact schemas, enums, bounds, BLAKE3, safe references, bounded collections, and redaction-safe text. r[mantle.machine_artifact_contracts.contract_vocabulary]
- [ ] [serial] Make the checker iterate contracted registry entries and bind owner, schema, contract, fixtures, consumer policy, and non-claims by BLAKE3. r[mantle.machine_artifact_contracts.registry_rail] r[mantle.machine_artifact_contracts.freshness]

## Phase 3: Initial high-value cohort

- [ ] [serial] Register and contract `BuildJsonReport`, `BuildPlanReport`, and `RoutePlanReport` public projections. r[mantle.machine_artifact_contracts.initial_cohort]
- [ ] [serial] Register and contract portable receipt bundle/verify/import and source-bundle plan/verify/offline-preflight projections. r[mantle.machine_artifact_contracts.initial_cohort]
- [ ] [serial] Register and contract Nickel export report/receipt plus the selected stable release/attestation handoff envelope. r[mantle.machine_artifact_contracts.initial_cohort]
- [ ] [parallel] Classify active remote-attempt, remote-observability, and Wasm-component machine surfaces without preempting their owning changes. r[mantle.machine_artifact_contracts.inventory]

## Phase 4: Fixtures and compatibility

- [ ] [parallel] Add Rust-serialized positive fixtures for every contracted family and prove schema plus generated Nickel contract acceptance. r[mantle.machine_artifact_contracts.fixtures]
- [ ] [parallel] Add negative fixtures for missing/unknown schema, unsupported version or enum, malformed digest, unsafe reference, oversized collection, unknown field, redaction leak, and family-specific cross-field disagreement. r[mantle.machine_artifact_contracts.fixtures]
- [ ] [serial] Add explicit converters and old/new fixtures only for versions the inventory marks as supported compatibility projections. r[mantle.machine_artifact_contracts.versioning]

## Phase 5: Verification

- [ ] [serial] Run checker self-tests, machine-contract generation/freshness, focused Rust producer tests, consumer compatibility fixtures, Cairn validation, and proposal/design/tasks gates. r[mantle.machine_artifact_contracts.freshness] r[mantle.machine_artifact_contracts.fixtures]
- [ ] [serial] Document that contract conformance does not prove build correctness, cache trust, reproducibility, release eligibility, attestation truth, or deployability. r[mantle.machine_artifact_contracts.runtime_boundary]
