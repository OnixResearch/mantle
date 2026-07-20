# Artifact-auth operational receipt

## ADDED Requirements

### Requirement: Non-production canary evidence is durably archived
r[mantle.artifact_auth_operational_receipt.canary_archive] Mantle MUST preserve its landed non-production artifact-auth canary as a self-contained public Cairn evidence bundle binding the exact product and artifact-auth revisions, harness source, real build observation, operational receipt, successful fresh-process replay, product-owned revocation transition, expected denial, typed manifest, and BLAKE3 inventory; the bundle MUST exclude private signing material and mutable secret state.

#### Scenario: Complete public canary bundle validates
GIVEN the exact public artifacts from the bounded Mantle canary run
WHEN the manifest, inventory, receipt identity, positive replay, and expected denial are checked
THEN every archived regular file is content-bound and the observed current-to-revoked transition remains reviewable without secret state

#### Scenario: Unsafe or incomplete archive fails closed
GIVEN a missing, drifted, symlinked, oversized, malformed, or secret-bearing candidate archive member
WHEN the canary archive is validated
THEN the evidence package is rejected before it can support a later admission review

### Requirement: Canary evidence does not grant authority
r[mantle.artifact_auth_operational_receipt.canary_authority] Mantle MUST label the archived canary as bounded non-production evidence and MUST keep `legacy_authoritative = true`, `standalone_authority_admitted = false`, and `rollback_available = true`; the archive MUST NOT claim remote trust discovery, revocation freshness, cache or build admission, registry publication, release eligibility, or production rollout.

#### Scenario: Passing canary remains observational
GIVEN a complete archived capture, replay, and revocation-denial record
WHEN an operator reviews the canary result
THEN legacy action-result behavior remains authoritative and a separate reviewed authority-admission change is still required
