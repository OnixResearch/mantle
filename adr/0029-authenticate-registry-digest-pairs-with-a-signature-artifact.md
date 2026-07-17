# ADR 0029: Authenticate registry digest pairs with a signature artifact

## Status

Accepted (2026-07-17)

## Context

ADR 0028 keeps an ordinary OCI image unchanged and preserves Mantle's exact layout/index/export evidence in a second subject-bound metadata manifest. Requiring both immutable manifest SHA-256 values detects tag and metadata drift relative to an operator handoff, but neither manifest authenticates who authorized the pair. Registry bearer credentials authenticate a request to one server; they are not durable artifact authority and must not become content identity.

A signature embedded in the metadata artifact cannot directly sign that artifact's final manifest digest without circularity. Signing only a local push receipt leaves registry pull dependent on an unspecified out-of-band transport. Mutating the ordinary image would invalidate the already-proven projection.

## Decision Drivers

- Authenticate both immutable manifest digests without mutating either manifest.
- Verify trust before content closure download or local admission.
- Keep registry routing, credentials, key paths, and local paths outside signed identity.
- Use explicit reviewable Nickel policy and deterministic Rust verification.
- Distinguish key names from full public-key identity and support bounded rotation/revocation/quorum.
- Preserve ordinary local OCI import as the only admission authority after trust succeeds.

## Decision

Mantle publishes a third deterministic OCI artifact manifest under `<image-tag>.mantle-signature`. Its subject is the exact image-manifest descriptor. A bounded annotation binds the metadata-manifest SHA-256 and trust domain. Its one role-annotated blob contains a deterministic signature document with a versioned signed statement and sorted detached Ed25519 signatures.

The signed statement domain-separates the schema and signature suite and binds the trust domain, image-manifest SHA-256, and metadata-manifest SHA-256. It excludes registry URL, repository, tags, bearer credentials, signing-key paths, trust-policy path, and local paths. A typed Nickel policy separately authorizes repositories for its trust domain, names required signers, supplies trusted public keys, supplies revoked full-key BLAKE3 identities, and sets a bounded minimum distinct-key threshold.

Push requires the policy and enough explicit signing keys to satisfy it. It uploads the signature document, publishes metadata then signature manifests, publishes the user-facing image tag last, and re-reads all three manifests by immutable digest before success.

Pull requires operator-supplied image, metadata, and signature manifest SHA-256 values plus the policy. It verifies all tag resolutions, signature artifact subject/annotation/blob linkage, statement identities, required signers, revocations, threshold, and detached signatures before downloading the image/metadata content closure. It tries every trusted key matching a signer name but counts only distinct verified full-key BLAKE3 identities. Only then may ordinary exact reconstruction/import run.

## Alternatives Considered

### Embed the signature in the metadata artifact

Rejected because signing the final metadata-manifest digest from inside that manifest is circular. Signing only its payload would not authenticate the exact immutable manifest pair exposed by the registry.

### Sign only the image manifest

Rejected because it permits substitution of Mantle's companion admission/export evidence while preserving the image.

### Sign only the local push receipt

Rejected because registry pull would need a separate, unspecified receipt transport and could not derive bounded remote signature evidence from the registry workflow itself.

### Bind registry URL/repository/tag in the signed statement

Rejected for the baseline because it would make byte-identical mirrors require resigning and would violate the established separation of registry routing from content identity. Repository authorization remains an explicit verifier-local policy decision.

### Use ambient key discovery or a transparency service

Deferred. The bounded baseline is deterministic and offline after inputs are supplied; it does not claim revocation freshness, global authority, or transparency.

## Consequences

- Trusted registry handoff requires three immutable manifest digests and one reviewed Nickel policy.
- Old unsigned push receipts cannot authorize trusted pull and are intentionally not silently upgraded.
- Mirrored bytes retain the same signed identity, but the verifier must explicitly authorize the mirror repository.
- Policy/key/signature collection bounds and distinct full-key counting prevent duplicate/name-based quorum inflation.
- Receipts record signature manifest, policy BLAKE3, trust domain, verified signer names, and verified public-key BLAKE3 identities without secret/path material.
- Signature verification authenticates the digest pair under supplied local policy. It does not prove registry authorization, transparency, revocation freshness, tag immutability, artifact correctness, arbitrary registry compatibility, kernel compatibility, bootability, deployability, or release eligibility.
