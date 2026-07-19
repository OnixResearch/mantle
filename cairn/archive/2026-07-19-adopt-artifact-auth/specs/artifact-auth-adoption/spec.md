## ADDED Requirements

### Requirement: Mantle adopts one immutable reviewed source

r[mantle.artifact_auth_adoption.source] Mantle MUST consume one immutable reviewed `artifact-auth` revision with aligned Cargo and Nix identities and MUST bind the Mantle mapping profile and checked projection from that same revision before implementation or cutover.

#### Scenario: Source identity is admissible

- GIVEN Cargo, Nix, the mapping profile, and its checked projection resolve revision `799459346d5416fbd7b9f55840a7371441b55afa`
- WHEN Mantle evaluates dependency admission
- THEN it SHALL reject floating, duplicate, mismatched, sibling-path, product-dependent, or license-incompatible source selections.

### Requirement: Mantle retains product authority

r[mantle.artifact_auth_adoption.authority] Mantle MUST retain OCI canonicalization, repository authorization, registry routing, credentials, signing, trust/currentness collection, cache/build admission, receipt composition, and release policy while treating standalone authentication as one bounded input.

#### Scenario: Authentication passes without product admission

- GIVEN a standalone signature and policy decision passes
- WHEN repository, cache, build, or release admission runs
- THEN Mantle MUST still require its product-owned checks and MUST NOT promote standalone success into product authority.

### Requirement: Cutover requires explained dual-run evidence

r[mantle.artifact_auth_adoption.cutover] Mantle MUST dual-run legacy and standalone paths over identical observations, classify every preimage, identity, decision, issue, and non-claim difference, reject unrelated-failure false parity, and preserve a bounded legacy rollback until standalone authority is explicitly admitted.

#### Scenario: Unexplained drift blocks cutover

- GIVEN any unexplained compatibility or source-identity difference
- WHEN Mantle evaluates cutover admission
- THEN the legacy path SHALL remain authoritative and the exact blocker SHALL be recorded without weakening current product gates.
