# ADR 0021: Accept Valence-authorized Trellis proof evidence

## Status

Superseded for the Kamacite envelope version by the canonical v2 profile cutover (2026-10-01); role and opaque-authority boundaries retained.

This ADR records the original v1 decision. The active release-core registration
and acceptance contract are now described in
[`docs/trellis-proof-release-sidecars.md`](../docs/trellis-proof-release-sidecars.md).
Kamacite has since published `kamacite.trellis-proof-evidence-profile.v2`;
v1 proof envelopes, including optional-mode inputs, are rejected rather than
aliased or interpreted as v2.

## Context

Kamacite profile `kamacite.trellis-proof-evidence-profile.v1` preserves the producer roles `recorded-only` and `formal-proof-candidate` without deciding downstream acceptance. Valence commit `27b8b212` archived `trellis-proof-evidence-profile` with an explicit `accepted_formal_proof` outcome. Its validator accepts that outcome only when verifier status is `passed`, policy acceptance is true, and the verification role is `property`.

Mantle must consume that authority without parsing Trellis source, proof IR, verifier logs, or canonical Preserves internals, and without reinterpreting existing `recorded_only` evidence as accepted.

## Decision

Mantle keeps the existing Kamacite v1 envelope profile and admits exactly these producer/validator role pairs:

- `recorded-only` with `recorded_only`;
- `formal-proof-candidate` with `recorded_only`;
- `formal-proof-candidate` with `property`.

Only `formal-proof-candidate` with `property` can satisfy required Trellis proof mode. The release result records disposition `accepted-formal-proof`, preserves both upstream roles, and retains the existing identity-linkage-only boundary. Every other pair fails closed.

The shell remains responsible for measuring the canonical envelope and Valence validation artifact and for extracting their public receipt identities. The pure release core compares those observations with the typed binding. Mantle does not re-run Valence semantics or infer proof truth from the role strings.

## Alternatives considered

### Promote `recorded_only`

Rejected because Valence still distinguishes recorded evidence from accepted property evidence.

### Invent a Kamacite v2 profile

Rejected because Kamacite has not published such a canonical schema. The accepted outcome is a Valence-owned validation result over the existing candidate envelope.

### Parse Valence or Trellis payloads in Mantle core

Rejected because Valence owns proof acceptance and Mantle owns only bounded release linkage.

## Consequences

Required mode can pass only with the exact accepted upstream role pair and matching measured artifacts. Optional recorded-only behavior remains compatible. This proves bounded release linkage to a Valence-accepted result; it does not prove verifier soundness, proof truth, semantic equivalence, whole-program correctness, downstream certification, or release eligibility by itself.

## Supersession

The active profile uses Kamacite's canonical v2 Preserves schema and
`kamacite.trellis-proof-evidence-profile.v2.compat-json` projection. Kamacite
requires typed source/proof IR/verifier-receipt identities, passed verifier for
`formal-proof-candidate`, assumption/dependency roots, property and requirement
IDs, and its three required non-claims. Valence alone accepts
`AcceptedFormalProof` with a passed, policy-accepted `Property` verification
and typed digests, spans, and scoped non-claim. Mantle measures canonical
bytes and Valence artifact bytes separately from the logical receipt hash;
it binds only opaque metadata and the original role-pair decision. Neither the
Trellis deterministic proof IR handoff nor synthetic Mantle contract fixtures
are an actual Valence-accepted property receipt.
