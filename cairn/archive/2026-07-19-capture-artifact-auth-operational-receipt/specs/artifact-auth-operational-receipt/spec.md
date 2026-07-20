# Artifact-auth operational receipt

## ADDED Requirements

### Requirement: Product-owned trust observation
r[mantle.artifact_auth_operational_receipt.trust] Mantle MUST derive standalone signer currentness from an explicit validated trust snapshot containing policy identity, observation time, validity bounds, full trusted verifying keys, and revoked full-key digests; key-name agreement or private-key possession alone MUST NOT produce `Current`.

#### Scenario: Trusted non-revoked key is current
GIVEN a full verifying key present in a valid Mantle trust snapshot and absent from its revocation set
WHEN Mantle derives the standalone signer observation
THEN it reports `Current` and binds the complete normalized trust snapshot with BLAKE3

#### Scenario: Unknown or revoked key fails closed
GIVEN a key absent from trusted full-key material or present in the revocation set
WHEN Mantle derives the standalone signer observation
THEN it reports `Unknown` or `Revoked` and standalone signing is denied

### Requirement: Durable receipt publication
r[mantle.artifact_auth_operational_receipt.persistence] Mantle MUST persist one bounded deterministic operational receipt under local action-result state only after exact standalone verification passes, and MUST bind the carrier refs, authorization ref, trust observation, compatibility flags, non-claims, and receipt content with BLAKE3.

#### Scenario: Receipt survives reopen
GIVEN a passing exact standalone shell result and valid trust observation
WHEN Mantle publishes and reopens the local receipt
THEN the reopened bytes validate to the same receipt identity before replay

#### Scenario: Malformed or replaced receipt is rejected
GIVEN truncated, malformed, path-substituted, or content-drifted receipt bytes
WHEN Mantle reloads the receipt
THEN validation fails before standalone evidence can be consumed

### Requirement: Fresh-context replay
r[mantle.artifact_auth_operational_receipt.replay] Mantle MUST recompute exact standalone verification and the product trust observation during replay, and MUST reject carrier, authorization, currentness, policy, validity-window, revocation, identity, outcome, or receipt drift.

#### Scenario: Fresh replay passes
GIVEN an unchanged action-result record, carrier, and valid current trust context
WHEN a persisted receipt is replayed after reopening state
THEN all recomputed evidence equals the persisted receipt

#### Scenario: Trust or carrier drift fails
GIVEN a revoked or replaced key, changed trust policy/window, or changed standalone carrier
WHEN the persisted receipt is replayed
THEN replay fails closed without changing action-result authority

### Requirement: Authority remains product-owned
r[mantle.artifact_auth_operational_receipt.authority] Mantle MUST keep `legacy_authoritative = true`, `standalone_authority_admitted = false`, and `rollback_available = true`; the operational receipt MUST state that remote trust discovery, revocation freshness, cache/build admission, registry publication, and release eligibility remain outside its claims.

#### Scenario: Passing receipt is non-authoritative
GIVEN a passing persisted and replayed receipt
WHEN compatibility is reported
THEN legacy action-result behavior remains authoritative and no standalone cutover occurs
