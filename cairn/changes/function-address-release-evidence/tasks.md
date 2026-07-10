## Phase 1: Function-address release evidence binding

- [x] [serial] Capture the Mantle function-address release evidence requirement delta before implementation. r[mantle.release_provenance.function_address_evidence.policy]
- [ ] [serial] Add release metadata/profile constants for function-address evidence role, schema, claim scope, digest fields, binary/source binding, and required non-claims. r[mantle.release_provenance.function_address_evidence.policy] r[mantle.release_provenance.function_address_evidence.binding]
- [ ] [serial] Implement pure validation over loaded release evidence rows while keeping file I/O, digest measurement, and bundle assembly in the shell. r[mantle.release_provenance.function_address_evidence.validation]
- [ ] [parallel] Add positive fixtures for optional-absent, optional-present, and required-present function-address evidence. r[mantle.release_provenance.function_address_evidence.fixtures.positive]
- [ ] [parallel] Add negative fixtures for missing sidecar, stale digest, wrong role/schema, wrong claim scope, binary/source mismatch, missing Valence receipt, missing non-claims, and overclaims. r[mantle.release_provenance.function_address_evidence.fixtures.negative]
- [ ] [parallel] Update operator docs and examples to show function-address evidence as opaque Valence/Kamacite evidence bound to release artifacts. r[mantle.release_provenance.function_address_evidence.docs]
- [ ] [serial] Run focused release-provenance tests, fixture checks, constants/profile drift checks, Cairn validation, and proposal/design/tasks gates before sync/archive. r[mantle.release_provenance.function_address_evidence.final_validation]
