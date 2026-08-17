## ADDED Requirements

### Requirement: OCI registry transport is bounded and explicit

r[kernel_bundle_oci.registry_transport] Mantle SHOULD publish admitted local OCI layouts through a bounded OCI Distribution-compatible transport that preserves exact image and Mantle metadata bytes, uses explicit endpoint and credential inputs, and emits success only after immutable manifest verification.

#### Scenario: Admitted layout publishes image and metadata

GIVEN a verified Mantle OCI layout, an explicit registry endpoint, repository, tag, and optional bearer-token file
WHEN registry push runs
THEN Mantle MUST upload or reuse the exact descriptor and metadata blobs, publish a companion metadata artifact whose subject is the image-manifest SHA-256, publish the user-facing image tag last, re-read both manifests by digest, and emit a receipt binding OCI SHA-256 and Mantle BLAKE3 identities
AND credentials, credential paths, local layout paths, redirects, ambient proxies, and external registry CLIs MUST NOT enter content identity or the receipt.

#### Scenario: Publication is interrupted or denied

GIVEN authentication is denied, one upload or manifest write fails, the registry returns a mismatched digest/location, or the final immutable verification fails
WHEN registry push runs
THEN Mantle MUST fail without emitting a successful receipt or claiming the user-facing tag was published
AND a later explicit rerun MAY reuse verified content-addressed blobs but MUST NOT claim exactly-once publication, transactional rollback, or upload resumption.

### Requirement: Registry pull reuses ordinary OCI admission

r[kernel_bundle_oci.registry_admission] Mantle MUST require expected image-manifest and subject-bound metadata-manifest SHA-256 values for registry pull, verify both immutable manifest identities and the complete metadata/content closure, reconstruct exact local layout bytes atomically, and pass that layout through ordinary OCI import before reporting admitted success.

#### Scenario: Immutable pull round-trips into fresh state

GIVEN registry image and companion tags resolve to the operator-supplied expected manifest SHA-256 values and the companion metadata artifact binds the same image subject
WHEN registry pull runs into absent layout/report destinations and fresh Mantle state
THEN Mantle MUST verify every downloaded manifest/blob descriptor, restore byte-identical `oci-layout`, `index.json`, export report, and descriptor blobs, atomically publish the local layout, and invoke ordinary OCI import
AND the pull receipt MUST bind the resolved OCI digests, restored layout/projection BLAKE3 values, and local import receipt identity.

#### Scenario: Tag, metadata, or content drifts

GIVEN either tag resolves to another digest, the companion subject differs, a metadata role is missing/duplicated, or any downloaded bytes mismatch their descriptor
WHEN registry pull runs
THEN Mantle MUST fail before final layout publication, CAS admission, import report publication, or successful pull receipt
AND it MUST NOT silently downgrade a formerly admitted Mantle layout to compatibility-only state.

### Requirement: Registry receipts are contracted machine artifacts

r[kernel_bundle_oci.registry_receipts] Stable registry push and pull reports MUST be Rust-owned machine artifacts registered with exact schemas, generated Nickel review contracts, version policy, positive and negative fixtures, BLAKE3 freshness, bounded transfer accounting, credential-mode redaction, and explicit non-claims.

#### Scenario: Registry report contract drifts

GIVEN a registry report DTO, schema, contract, fixture, inventory policy, digest role, credential field, or non-claim changes without regeneration and review
WHEN the machine-artifact contract rail runs
THEN it MUST fail before packaging and identify the stale or unsafe artifact.

### Requirement: Registry publication has positive and negative evidence

r[kernel_bundle_oci.registry_verification] The registry lane MUST include deterministic pure-core, HTTP-shell, public CLI, fresh-state admission, gallery, schema/contract, documentation, and lifecycle evidence covering authenticated round trip, immutable digest resolution, digest-role separation, tag drift, metadata/blob tampering, denied credentials, interrupted publication, content-addressed retry, redaction, and non-claims.

#### Scenario: Registry workflow is ready to publish

GIVEN maintainers intend to advertise and archive the registry-backed OCI workflow
WHEN closeout validation runs
THEN focused positive/negative checks, first-party quality, Tiger Style, machine-contract validation, dependency audit, documentation drift checks, Tracey coverage, and Cairn proposal/design/tasks gates MUST pass before sync, archive, commit, and push
AND evidence MUST retain the bounded compatibility claim and MUST NOT infer registry trust, tag immutability, signature verification, exactly-once publication, kernel compatibility, bootability, deployability, or release eligibility.
