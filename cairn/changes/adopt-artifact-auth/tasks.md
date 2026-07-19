## Phase 1: Consumer baseline and source admission

- [x] [serial] I1 Capture current Mantle OCI/action-result signature preimages, key identities, threshold/revocation decisions, issue ordering, retained authority, and positive/negative fixtures; compare them with the immutable published Mantle mapping profile. r[mantle.artifact_auth_adoption.source] r[mantle.artifact_auth_adoption.authority]
- [x] [serial] I2 Pin Cargo and Nix to `ssh://git@github.com/OnixResearch/artifact-auth.git` revision `799459346d5416fbd7b9f55840a7371441b55afa`, generate locks only with owning tools, and reject floating, duplicate, mismatched, sibling-path, product-dependent, or license-incompatible sources. r[mantle.artifact_auth_adoption.source]

## Phase 2: Adapter and cutover

- [x] [depends:mantle.artifact_auth_adoption.source] I3 Add a pure adapter for OCI subjects/parents, purpose/domain, raw full-key identity, compatibility key identity, distinct-key threshold, required signer labels, and supplied revocation/currentness observations while keeping filesystem, registry, credential, signing, build, and release effects in Mantle shells. r[mantle.artifact_auth_adoption.authority]
- [x] [depends:mantle.artifact_auth_adoption.authority] I4 Dual-run legacy and standalone paths over identical positive and tamper observations, classify every difference, reject unrelated-failure false parity, and retain legacy authority plus rollback until admission. r[mantle.artifact_auth_adoption.cutover]
- [x] [depends:mantle.artifact_auth_adoption.cutover] I5 Admit or reject cutover from durable compatibility evidence, update operator migration/rollback documentation, and remove duplicate canonical ownership only after the bounded rollback period. r[mantle.artifact_auth_adoption.cutover]

## Phase 3: Verification

- [x] [parallel] V1 Add positive tests for valid OCI pair binding, distinct full-key threshold, required labels, and bounded non-claims plus negative tests for wrong OCI digest/domain/purpose, duplicate-label inflation, revoked/unknown keys, malformed signatures, registry/repository authority promotion, and weakened non-claims. r[mantle.artifact_auth_adoption.authority] r[mantle.artifact_auth_adoption.cutover]
- [ ] [serial] V2 Run focused signature/action-result tests, exact-source checks, first-party quality, Tiger Style, Cargo/Nix builds, lifecycle validation/gates, accepted-spec sync, and archive with exact bounded evidence. r[mantle.artifact_auth_adoption.source] r[mantle.artifact_auth_adoption.cutover]
