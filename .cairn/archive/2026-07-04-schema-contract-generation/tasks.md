# Tasks

## Contract

- [x] [serial] Define the machine-schema contract rail, selected JSON surface inventory, generated contract freshness rule, unsupported-schema behavior, and bounded non-claims. r[verification_evidence.machine_schema_contracts]

## Implementation

- [x] [serial] Add schema inventory metadata for selected Mantle JSON surfaces and their schema owners. r[verification_evidence.machine_schema_contracts]
- [x] [serial] Add deterministic JSON Schema to Nickel contract generation or snapshot checking for the selected surfaces. r[verification_evidence.machine_schema_contracts]
- [x] [serial] Add contract validation plumbing that can run selected positive and negative fixtures without depending on runtime remote schema conversion. r[verification_evidence.machine_schema_contracts]

## Verification

- [x] [serial] Add positive fixtures for selected build report, doctor report, project diagnostic, release/proof manifest, or demo summary payloads. r[verification_evidence.machine_schema_contracts]
- [x] [serial] Add negative fixtures for missing required fields, type mismatches, invalid schema versions, malformed enum values, and stale generated contracts. r[verification_evidence.machine_schema_contracts]
- [x] [serial] Run `nix run path:/home/brittonr/git/cairn#cairn -- validate --root .` and proposal/design/tasks gates before implementation tasks are marked complete. r[verification_evidence.machine_schema_contracts]

Evidence: `nix develop -c cargo -Zscript scripts/check-machine-schema-contracts.rs`, `nix develop -c cargo -Zscript scripts/check-machine-schema-contracts.rs --self-test`, and Cairn proposal/design/tasks gates passed for this change.
