# Release source-review evidence

Mantle can bind cryptographically attributable source-review decisions to the exact released source identity. Mantle consumes a stable externally produced review attachment. It never creates reviewer statements, runs review workflow, or trusts producer status fields.

## Roles

- **Cairn** owns review workflow, obligations, dispositions, and signed verifier decisions.
- **Valence** owns cross-project evidence identity. The attachment binds one Valence evidence BLAKE3.
- **Mantle** verifies attachment identity, exact source linkage, signatures, reviewer-key policy, and role separation only.

## Attachment

One attachment is a `mantle.source-review.attachment.v1` JSON document:

- `subject`: the exact release source archive BLAKE3, an optional immutable source revision, the reviewed Cairn claim root BLAKE3, and the review policy BLAKE3 the statements were produced under.
- one or more `approvals`: each carries the statement domain, reviewer label, full Ed25519 public key, detached signature, disposition (`approved` or `needs-revision`), and key generation.
- `producer_obligation_id` and `producer_disposition`: producer facts recorded but never authoritative.
- `valence_evidence_blake3`: identity of the Valence evidence record.
- required non-claims and a canonical `attachment_blake3` identity recomputed by Mantle.

The machine-readable projection is the contracted `release.source-review-attachment` surface in `schemas/machine-contracts/`.

## Commands

Bundle an attachment without modifying its external authority fields:

```bash
mantle release create ... --source-review-attachment review.json
```

Verify with an explicit reviewed-source policy:

```bash
mantle release verify <bundle> \
  --reviewed-source required \
  --review-preset stagex-two-reviewer \
  --trusted-reviewer alice:<base64-public-key> \
  --trusted-reviewer bob:<base64-public-key> \
  --review-claim-root <claim-root-blake3>
```

Other inputs:

- `--source-review <path>` overrides the bundled member.
- `--revoked-reviewer label:<base64>` records revoked keys in policy.
- `--review-author-key <base64>` excludes the change author from approval counting.
- `--review-threshold <n>` replaces the named preset with an explicit distinct-reviewer count.
- `--review-producer-id <id>` overrides the default `cairn-verification-obligation` producer identity.

## Policy semantics

- Generic releases report source review as `not-required` when no reviewed-source policy is selected. Missing review evidence never blocks a generic release.
- Optional evidence that is present must verify: when a bundle carries an attachment, the operator must pass `--trusted-reviewer` keys and `--review-claim-root`, or verification fails closed with guidance.
- The `stagex-two-reviewer` preset requires two distinct authorized reviewer full public-key identities. Duplicate keys, duplicate labels, one key under multiple labels, unknown keys, revoked keys, excluded author identities, and `needs-revision` dispositions cannot satisfy the threshold.
- Approvals bind the exact source archive digest, claim root, and policy digest. Any mismatch fails with a deterministic reason code: `stale-source-subject`, `stale-claim-root`, or `policy-mismatch`.
- Signatures verify through the pinned Artifact Auth Ed25519 boundary over canonical statement bytes.
- Source-review approvals, release signatures, and build-witness attestations are domain-separated. A build-witness statement in a review attachment fails with `role-confusion`, and review approvals never satisfy build-witness policy.

## Producer workflow

1. The change author requests review through the Cairn verification-obligation workflow.
2. Authorized reviewers sign approval statements over the canonical subject (source archive digest, claim root, review policy digest).
3. The producer assembles the attachment with the Valence evidence identity and required non-claims, computes the canonical `attachment_blake3`, and publishes it.
4. The release operator bundles the attachment and selects the reviewed-source policy at verification time.

## Key and revocation inputs

Reviewer public keys are operator policy inputs in `label:base64` form. Currentness classification comes from the same policy: `--trusted-reviewer` keys are current and `--revoked-reviewer` keys are revoked. Key rotation and revocation freshness remain operator responsibility; Mantle does not discover keys or fetch revocation lists.

## Non-claims

A satisfied reviewed-source policy proves only that the configured distinct authorized reviewer keys signed approval statements bound to the exact released source and policy. It does not prove source correctness, review completeness, reviewer competence, build correctness, reproducibility, witness independence, deployment safety, or pull-request approval.
