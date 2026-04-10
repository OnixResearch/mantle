## Phase 1: Native model and hashing

- [x] Define the native attestation schema, canonical ordering rules, and schema versioning
- [x] Implement a pure canonicalization + digest core for artifact, closure, and project attestations
- [x] Add deterministic tests proving traversal-order independence and stable digests

## Phase 2: Build/store integration

- [x] Persist per-output artifact attestations as part of successful build finalization
- [x] Attach substitution results to the same artifact-attestation model
- [x] Add store APIs for retrieving artifact attestations by logical store path and aggregate attestations by rooted selection

## Phase 3: Nickel and project claims

- [ ] Extend builder-layer Nickel contracts with optional provenance claims metadata
- [ ] Fold `crunch.lock`, mirrors, and patch records into source and project attestations
- [ ] Add tests showing claim changes do not affect derivation hashes by default

## Phase 4: CLI and verification

- [ ] Add `crunch attest` commands for show, closure, verify, diff, and project views
- [ ] Extend `crunch --json build` with references to generated attestations
- [ ] Add end-to-end tests for build, substitution, project, and verification workflows
