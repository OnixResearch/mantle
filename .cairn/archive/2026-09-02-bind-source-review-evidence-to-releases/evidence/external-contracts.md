# External contracts for source-review evidence (I1)

Recorded stable contracts consumed by this change. No placeholder or draft-only contracts were used.

## Cairn producer contract

- Repository: `/home/brittonr/git/OnixResearch/cairn`, native Cairn layout.
- Capability: `verification-obligation-radicle-cob` specification at `.cairn/specs/verification-obligation-radicle-cob/spec.md`.
- Typed profile: custom type `xyz.onixresearch.verification-obligation` with stable obligation ID, structured claim, subject identities, verification criteria, role sets, evidence links, findings, and lifecycle facts (`r[cairn.verification_obligation_cob.profile]`).
- Authority separation: submitters cannot discharge obligations (`r[cairn.verification_obligation_cob.authority.denied]`); discharge binds to the current conflict-free claim root (`r[cairn.verification_obligation_cob.discharge]`, `r[cairn.verification_obligation_cob.claim_revision]`).
- Receipts distinguish signature validity, configured role authorization, workflow discharge, evidence identity, evidence truth, and proof soundness (`r[cairn.verification_obligation_cob.receipts.discharge]`).
- Semantic-review ledger receipts gate release readiness separately in `cairn/semantic-review` specs; readiness consumes review evidence without owning review workflow.

Baseline validation: `cairn validate --root .` on the Cairn repository reports `"valid": true` with no findings (see `baseline-cairn-validate.log`).

## Artifact Auth signature contract

- Source: `ssh://git@github.com/OnixResearch/onix-artifact.git`, pinned revision `c932138d880ddf4c2967f4c024b489b5c0022bf1` (same pin already consumed by `crunch-build` and `crunch-action-result-core`).
- Core crate `artifact_auth_core` (no-std): `STATEMENT_SCHEMA_V1`, `POLICY_SCHEMA_V1`, `PREIMAGE_DOMAIN_V1`, canonical length-delimited statement bytes, `statement_identity`, `policy_identity`, `evaluate_authentication` with distinct verified-key threshold, trusted-key currentness (`Current`, `VerificationOverlap`, `Superseded`, `Revoked`), generation checks, and required non-claims.
- Crypto crate `artifact_auth_ed25519`: `verify_statement` returning deterministic `CryptographicObservation` with stable failure classes, and `public_key_identity` deriving the ed25519-public-key BLAKE3 profile.
- Mantle review statements map into `ArtifactStatement` with review-owned `AuthenticationScope` (domain `mantle.release.source-review.v1`, purpose `approve-release-source`, profile `mantle-release-source-review-v1`, subject/parents/verifier_context bound to the release source, claim root, policy digest, and Valence evidence identity).

## Valence identity contract

- The attachment binds one Valence evidence BLAKE3 as the statement `verifier_context`. Valence owns evidence identity and linkage semantics; Mantle records and covers it with signatures but does not interpret the evidence record.

## Role and source-subject contracts

- Reviewer role: full Ed25519 public-key BLAKE3 identity plus producer identity plus label, counted once per distinct key, current per operator policy.
- Release-signer and build-witness roles use domain-separated statements; review approvals never satisfy witness quorum and witness attestations never satisfy review policy.

## Compatibility

- Generic release manifests serialize identically when no attachment is present (`Option` with `skip_serializing_if`).
- The verify JSON report gains one `source_review` field inside the compatibility-class `release.evidence-reports` family.
- The machine-contract registry gains the contracted `release.source-review-attachment` surface; no existing surface changed semantics.

## Non-claims

These contracts give Mantle attributable review action over the exact source identity only. They do not establish reviewer competence, review completeness, source correctness, build correctness, reproducibility, or release eligibility.
