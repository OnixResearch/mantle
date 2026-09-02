# ADR 0111: Bind source-review evidence to releases through Artifact Auth

## Status

Accepted

## Context

Mantle release evidence binds source archives, proofs, outputs, and witnesses, but not cryptographically attributable source-review decisions. Cairn owns the verification-obligation workflow with signed verifier roles and bounded discharge. Valence owns evidence identity. Mantle must consume reviewed external artifacts without becoming a code-review system or trusting producer status strings.

The optional-versus-required decision matters: making review mandatory for every release would couple generic releases to an external workflow that is not always present.

## Decision

Mantle consumes one stable `mantle.source-review.attachment.v1` artifact.

- The pure core in `crunch-release-core/src/source_review.rs` validates attachment structure, canonical identity, exact source linkage, reviewer-key policy, role separation, and distinct approval counting. Cryptographic observations are supplied by the shell.
- Signature verification goes through the pinned Artifact Auth Ed25519 boundary over canonical statement bytes; every review statement maps into an Artifact Auth statement with the review domain, purpose, and profile.
- Review policy is optional by default. Generic releases report `not-required` when no attachment exists. Present optional evidence must verify against operator-supplied reviewer keys and the current claim root, or verification fails closed.
- The named `stagex-two-reviewer` preset defines the two-distinct-reviewer threshold in typed policy. Threshold counting deduplicates by full public-key BLAKE3 identity and rejects duplicate policy keys, unknown keys, revoked keys, excluded author keys, `needs-revision` dispositions, and build-witness domain statements.
- The attachment binds the exact source archive digest, claim root, and review policy digest. Operator policy changes (including author exclusion) change the policy identity and therefore require attachments produced under that same policy.
- The machine-readable projection is the contracted `release.source-review-attachment` surface with generated Nickel contract, positive fixture, and negative fixture set.

## Consequences

- Existing generic release verification gains a `source-review` decision contributor and JSON field; manifests without an attachment serialize identically and remain compatible.
- Release bundles can carry one attachment at `source-review/attachment.json`, measured and remeasured like every other bundle member.
- Producer status fields and reviewer display names never authorize a decision.
- A satisfied policy proves attributable review action over the exact source only; it proves nothing about source correctness, review quality, build correctness, or release eligibility.
- Author exclusion is part of the policy identity, so changing the exclusion requires review statements produced under the new policy.
