## Context

The accepted registry workflow publishes an ordinary OCI image plus a subject-bound companion metadata artifact and requires both immutable manifest digests on pull. This detects mutable-tag and metadata substitution relative to an operator handoff, but the handoff remains unsigned. Existing Mantle key handling already provides Ed25519 detached signatures and key parsing, while Nickel is the repository default for reviewed policy/configuration.

## Decisions

### Decision: Sign a third immutable OCI artifact over the image/metadata digest pair

**Choice:** Publish `<tag>.mantle-signature` as an OCI artifact whose subject is the image manifest and whose single role-annotated blob is a deterministic signature document. The signed statement contains a versioned domain separator, operator policy trust domain, image-manifest SHA-256, and metadata-manifest SHA-256. The signature artifact manifest also binds the metadata digest and trust domain in bounded annotations.

**Rationale:** A third artifact authenticates the already-proven pair without mutating the ordinary image or metadata bytes and avoids treating a local receipt file as remotely available authority. Pull can require all three immutable manifest digests and verify the signature artifact before content download or admission.

### Decision: Keep registry routing outside signed identity

**Choice:** The signed statement does not contain registry URL, repository, tag, credential mode, key path, or local path. The Nickel policy separately constrains an exact bounded set of allowed repositories for the trust domain.

**Rationale:** This preserves ADR 0028's routing/content separation and permits byte-identical mirroring. Cross-repository use still fails closed unless the verifier's reviewed local policy authorizes that repository.

### Decision: Use explicit typed Nickel policy with deterministic Rust validation

**Choice:** The shell evaluates one explicit `.ncl` policy into a Rust DTO. The pure core validates schema/version, trust-domain and repository bounds, required signer names, trusted public keys, revocation BLAKE3 digests, and signature threshold. Canonical normalized policy bytes determine a BLAKE3 policy digest recorded in receipts. Runtime trust decisions use only validated in-memory facts.

**Rationale:** Nickel provides reviewable configuration contracts; Rust owns cryptographic verification and deterministic receipt semantics. No ambient trust store, clock, network key discovery, or registry authorization state enters the decision.

### Decision: Verify before content closure download and ordinary admission

**Choice:** Pull resolves and verifies the image, metadata, and signature manifests by operator-supplied immutable digest, downloads only the bounded signature document, validates the metadata linkage and policy, verifies required non-revoked Ed25519 signatures, and only then downloads image/metadata content blobs. Any trust failure leaves layout, import report, pull receipt, and CAS admission absent.

**Rationale:** Verifying after import would permit untrusted bytes to cross the admission boundary and would make failure cleanup harder to audit.

### Decision: Support bounded threshold verification without key-name shortcuts

**Choice:** A signature document carries a bounded sorted list of detached signatures. Verification tries every trusted key with the matching signer name, deduplicates by full public-key BLAKE3 digest, rejects revoked key digests, requires every named required signer, and enforces a bounded minimum distinct-key threshold.

**Rationale:** Key names are routing labels, not key identity. Full-key matching supports controlled rotation while preventing duplicate signatures or duplicate names from inflating quorum.

## Risks / Trade-offs

- Requiring a third immutable digest increases operator handoff data and makes old unsigned registry receipts insufficient for trusted pull.
- The tag convention is still not OCI Referrers API discovery and does not claim arbitrary-registry compatibility.
- Policy revocation is verifier-local and has no freshness, transparency-log, or global distribution guarantee.
- Signature verification authenticates the digest pair under supplied policy; it does not prove registry authorization, tag immutability, artifact correctness, kernel compatibility, bootability, deployability, or release eligibility.
