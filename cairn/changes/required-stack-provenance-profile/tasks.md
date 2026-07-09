## Tasks

- [ ] [serial] Define the stack-release profile contract with required stack provenance mode. r[mantle.release_provenance.stack_profile.required]
- [ ] [serial] Centralize or generate stack-provenance role, schema, claim-scope, and non-claim constants. r[mantle.release_provenance.stack_profile.constants]
- [ ] [serial] Add positive fixtures for stack-release bundles with matching sidecar, Valence graph report, binary link, and non-claims. r[mantle.release_provenance.stack_profile.positive]
- [ ] [serial] Add negative fixtures for absent evidence, stale digest, wrong role/schema, wrong binary link, unsupported claim scope, and weakened non-claims. r[mantle.release_provenance.stack_profile.negative]
- [ ] [serial] Update operator docs to distinguish generic optional stack provenance from required stack release profiles. r[mantle.release_provenance.stack_profile.docs]
- [ ] [serial] Run focused release evidence tests, schema/contract checks, and Cairn validation/gates for this change. r[mantle.release_provenance.stack_profile.validation]
