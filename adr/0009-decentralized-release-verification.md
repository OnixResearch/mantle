# ADR 0009: Decentralized release verification separates technical and social trust

## Status

Proposed (2026-04-17)

## Context

Crunch now has stronger release evidence than a bare binary release:

- `crunch release create` packages the tracked-worktree source archive, release
  binary or binaries, full proof bundle, and prerequisite inventory
- `crunch release verify` checks canonical manifest bytes, artifact digests, and
  proof linkage using bundle-local contents only
- the self-hosting proof gives a concrete stage0 -> stage1 -> stage2 fixed-point
  result for one build universe

That is useful packaged integrity evidence, but it is not yet decentralized
release verification. A single builder or maintainer can still produce a fully
self-consistent bundle. Stronger trust claims need at least two distinct layers:

1. **Technical verification**: canonical release attestations, witness result
   formats, digest-comparison rules, signature verification, and CLI behavior.
2. **Social trust policy**: signer and rebuilder roles, independence rules,
   quorum thresholds, revocation policy, and publication expectations.

If those layers are mixed into one artifact schema, every policy change risks
invalidating technical identities, and every technical review turns into an
organization-specific governance debate.

## Decision

Crunch will model decentralized release verification as two cooperating systems:

### Technical verification layer

The technical layer defines machine-checkable artifacts and algorithms:

- canonical release attestations that bind a release identifier to release
  evidence and published output digests
- canonical witness attestations for independent rebuilders
- signature and digest verification rules
- trust-tier computation inputs
- CLI commands for inspecting and verifying attestations and witness sets

This layer MUST remain deterministic and reusable across organizations.
Artifact digests must not embed mutable local policy.

### Social trust policy layer

The social layer defines who is allowed to attest and what policy is required to
promote a release beyond self-proof-only status:

- role definitions such as builder, reviewer, releaser, and witness rebuilder
- quorum thresholds and independence requirements
- signer-key governance, revocation, and dispute handling
- publication requirements for release status and witness material

This layer MUST consume technical artifacts by stable digest or canonical
attestation identity. It MAY evolve without changing technical artifact hashes.

## Consequences

- Crunch gets a path from "internally consistent release bundle" toward
  "independently witnessed release" without over-claiming trust in one step.
- Different organizations can share the same release-attestation and
  witness-attestation formats while enforcing different trust policies.
- The verifier can report two truths at once: technical agreement and policy
  sufficiency.
- Operator output becomes more complex because a release may be technically
  valid but socially insufficient.
- Additional release machinery is required: witness collection, signer policy,
  and publication workflow.

## Alternatives Considered

### One merged technical-plus-social protocol

Rejected. It would bake mutable organizational policy into technical artifact
identity and make reuse harder.

### Social process only, without canonical witness artifacts

Rejected. Human ceremony alone does not give machine-checkable decentralized
verification.

### Technical witness artifacts only, without explicit social policy

Rejected. "N signatures" is not meaningful without role, independence, and
revocation rules.
