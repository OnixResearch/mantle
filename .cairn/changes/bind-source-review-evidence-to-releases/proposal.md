# Change: Bind source-review evidence to releases

## Why

Mantle release evidence binds source archive bytes, build proofs, outputs, and witness sidecars. It does not bind cryptographically attributable source-review decisions to the exact released source identity.

Cairn already owns semantic-review and lifecycle decisions, and its active verification-obligation work defines signed verifier roles and bounded discharge decisions. Valence owns cross-project evidence identity and linkage. Mantle must consume those reviewed external artifacts without becoming a code-review system or treating build witnesses as source reviewers.

## What Changes

- Define a bounded source-review attachment for release evidence that binds the exact release source BLAKE3, review claim root, review policy digest, reviewer full-key identities, signed approval references, disposition, and non-claims.
- Add a generic release mode where source-review evidence is optional and its absence is reported as `not-required`.
- Add an opt-in reviewed-source release policy that requires an explicit distinct-reviewer threshold. The StageX-inspired preset uses a named two-reviewer threshold.
- Recompute the release source identity and independently verify the supplied review attachment before it can satisfy the selected release policy.
- Keep source-review approval, release signing, and build-witness evidence as separate roles and decisions.
- Preserve generic release, bootstrap parity, StageX no-quorum verification, and optional build-witness behavior when reviewed-source policy is not selected.

## Non-Goals

- Running reviewers, managing pull requests, interpreting review prose, or deciding source correctness inside Mantle.
- Treating a signed approval as proof that a review was competent or complete.
- Letting source-review signatures satisfy build-witness quorum or letting build-witness signatures satisfy source-review policy.
- Inventing a Mantle-only review protocol before Cairn and Valence publish stable producer and identity profiles.
- Requiring source-review evidence for every generic build or release.

## Dependencies

- Cairn semantic-review receipts and the active `add-verification-obligation-radicle-cob` change, or a stable successor that supplies signed reviewer authority and discharge facts.
- A Valence evidence profile that preserves the exact Cairn review identity, signature-verification role, source subject, policy, and non-claim boundary.
- `artifact-auth-ed25519` or the stack-selected independent verifier for canonical detached-signature checks.

## Impact

- **Affected spec:** `verification-evidence`
- **Affected code:** release-evidence schemas, source-review adapter core, release verification policy, CLI inputs, reports, machine contracts, documentation, and tests
- **Testing:** external-profile compatibility; exact source binding; positive optional and required modes; negative stale, duplicate, revoked, role-confused, tampered, and insufficient approvals; Cairn gates
