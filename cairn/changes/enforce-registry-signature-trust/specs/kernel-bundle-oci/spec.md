## ADDED Requirements

### Requirement: Registry digest pairs require explicit signature trust

r[kernel_bundle_oci.registry_signature_trust] Mantle MUST authenticate a registry image/metadata manifest digest pair under an explicit typed trust policy before a pulled content closure can reach local OCI publication or admission.

#### Scenario: Trusted publication binds the immutable pair

GIVEN an admitted local OCI layout, a validated Nickel trust policy authorizing the target repository, and enough explicit non-revoked signing keys to satisfy policy
WHEN registry push runs
THEN Mantle MUST publish a deterministic signature artifact over the exact image-manifest and metadata-manifest SHA-256 pair before publishing the user-facing image tag
AND the push receipt MUST bind the immutable signature-manifest digest, trust-domain, policy BLAKE3, verified signer names, and public-key BLAKE3 identities without exposing key paths or key material.

#### Scenario: Trusted pull verifies before content admission

GIVEN operator-supplied immutable image, metadata, and signature manifest digests plus a validated policy authorizing the repository
WHEN registry pull runs
THEN Mantle MUST verify signature artifact subject/linkage, trust domain, required signers, distinct-key threshold, revocations, and detached Ed25519 signatures before downloading the image/metadata content closure
AND only a verified pair MAY proceed through exact layout reconstruction and ordinary OCI import.

#### Scenario: Policy or signer is untrusted

GIVEN a policy is malformed, authorizes another repository or trust domain, lacks a required signer, contains an invalid key, names a revoked key digest, or cannot satisfy its threshold
WHEN push or pull evaluates trust
THEN Mantle MUST fail closed before successful publication or admission
AND it MUST NOT emit a successful receipt, admitted layout, import report, or trust-verification claim.

#### Scenario: Signature linkage or immutable digest drifts

GIVEN the signature tag resolves to another manifest, its subject or metadata annotation differs, its signature document names another digest pair/domain, or detached signature bytes are invalid
WHEN pull verifies the registry handoff
THEN Mantle MUST reject the handoff before content closure download and admission
AND it MUST NOT downgrade to unsigned transport or treat registry possession, bearer credentials, or matching tags as trust.

#### Scenario: Duplicate names or signatures cannot inflate threshold

GIVEN multiple trusted keys share a signer name or a signature document repeats one signature/key
WHEN signature threshold is evaluated
THEN Mantle MUST try each full trusted key identity but count only distinct verified public-key BLAKE3 identities
AND duplicate names, duplicate signatures, revoked keys, and untrusted keys MUST NOT satisfy required-signer or minimum-signature policy accidentally.

#### Scenario: Trust claim remains bounded

GIVEN signature verification succeeds
WHEN a receipt, gallery, runbook, or lifecycle summary reports the result
THEN the claim MUST be limited to authentication of the immutable digest pair under the supplied local policy
AND it MUST NOT infer registry authorization, transparency, revocation freshness, tag immutability, arbitrary-registry compatibility, artifact correctness, kernel compatibility, bootability, deployability, or release eligibility.

## MODIFIED Requirements

### Requirement: Registry publication has positive and negative evidence

r[kernel_bundle_oci.registry_verification] The registry lane MUST include deterministic pure-core, HTTP-shell, public CLI, fresh-state admission, gallery, schema/contract, documentation, and lifecycle evidence covering authenticated round trip, immutable digest resolution, digest-role separation, signature-policy verification, tag drift, metadata/signature/blob tampering, unknown and revoked keys, denied credentials, interrupted publication, content-addressed retry, redaction, and non-claims.

#### Scenario: Registry workflow is ready to publish

GIVEN maintainers intend to advertise and archive the registry-backed OCI workflow
WHEN closeout validation runs
THEN focused positive/negative checks, first-party quality, Tiger Style, machine-contract validation, dependency audit, documentation drift checks, Tracey coverage, and Cairn proposal/design/tasks gates MUST pass before sync, archive, commit, and push
AND evidence MUST retain the bounded compatibility and signature-policy claim scopes and MUST NOT infer registry authorization, transparency, revocation freshness, tag immutability, exactly-once publication, arbitrary registry compatibility, kernel compatibility, bootability, deployability, or release eligibility.
