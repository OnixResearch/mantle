## ADDED Requirements

### Requirement: Exact standalone statement verification

r[mantle.artifact_auth_shell.exact_verification] Mantle MUST map each validated action-result signer to the exact canonical `artifact-auth.statement.v1` preimage, sign those bytes separately from the legacy action-result preimage, and independently verify them through `artifact-auth-ed25519` at revision `799459346d5416fbd7b9f55840a7371441b55afa`.

#### Scenario: Current authorized action-result signer produces valid standalone evidence

- GIVEN an admitted legacy action-result decision, its exact signed record, a current matching Mantle Ed25519 key, and valid currentness evidence
- WHEN the shell maps, signs, and evaluates the standalone statement
- THEN the standalone cryptographic observation SHALL verify over exact canonical bytes and SHALL bind recomputed statement, full public-key, signature, action-result, output-parent, and publication-policy identities.

### Requirement: Product authorization remains mandatory

r[mantle.artifact_auth_shell.authorization] Mantle MUST reject standalone signing unless the legacy action-result decision is admitted, the exact record signature verifies under the same full key and producer identity, and supplied key currentness permits signing.

#### Scenario: Detached action-result signature is not standalone proof

- GIVEN a valid legacy signature over the action-result `result_ref`
- WHEN those signature bytes are reused as if they covered the standalone statement
- THEN independent standalone verification SHALL reject them and Mantle SHALL retain the legacy signature only as external authorization evidence.

### Requirement: Bounded evidence fails closed

r[mantle.artifact_auth_shell.evidence] Mantle MUST recompute all carrier identities, preserve stable cryptographic failure classes, and report legacy authority, standalone non-authority, rollback, currentness, and external signing-authorization observations without including secret key material.

#### Scenario: Carrier or identity drift is observed

- GIVEN a changed statement, public key, signature, signature encoding, authorization carrier, or currentness reference
- WHEN the shell evaluates the carrier
- THEN evaluation SHALL reject malformed identity drift or return an explicit failed standalone observation without promoting it into a product gate.

### Requirement: Adversarial parity is explicit

r[mantle.artifact_auth_shell.adversarial] Mantle MUST test valid signatures, tampering, wrong standalone preimages, wrong keys, malformed signatures, key-label/full-key drift, revoked or unknown currentness, authorization failure, decision drift, and unrelated-failure false parity.

#### Scenario: Legacy and standalone failures have unrelated causes

- GIVEN legacy rejection and standalone rejection caused by disjoint policy facts
- WHEN compatibility is compared
- THEN the case SHALL be blocked as unrelated rejection causes rather than counted as parity.

### Requirement: Standalone authority is not admitted by the pilot

r[mantle.artifact_auth_shell.authority] Mantle MUST keep action-result, PathInfo, OCI, repository, registry, credential, key lifecycle, build/cache, receipt, and release gates authoritative, with rollback available and standalone authority unadmitted until a separate cross-consumer authority change proves operational readiness.

#### Scenario: All deterministic pilot tests pass

- GIVEN exact signing and verification evidence from the pilot
- WHEN readiness is evaluated across Mantle, Molten, and Valence
- THEN test success alone SHALL NOT admit standalone authority or claim current operational trust, currentness freshness, publication, persistence, deployment, or release eligibility.
