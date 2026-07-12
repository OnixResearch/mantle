# ADR 0019: Treat Preserves function-address receipts as opaque canonical sidecars

## Status

Accepted

## Context

Kamacite can carry a function-address receipt as canonical Preserves bytes while Valence exposes a separate logical validation receipt and JSON may remain available as a compatibility projection. Mantle must bind those artifacts to release source and binary identities without parsing Preserves values or promoting upstream semantic claims.

The generic opaque-evidence binding already commits to a canonical envelope, upstream validation artifact, source, binary, policies, optional compatibility projections, claim scope, and non-claims. The Cairn-facing function-address receipt already distinguishes bundle-file digests from upstream logical receipt identities.

## Decision Drivers

- Keep canonical Preserves bytes authoritative without adding a Preserves parser to Mantle.
- Detect stale Valence logical identities and stale JSON projections, not just stale file bytes.
- Preserve optional and required function-address policy modes.
- Reuse the generic opaque-evidence boundary instead of creating another manifest model.
- Keep the Cairn-facing `mantle.function-address-binding.v1` output shape stable.
- Bound every read and sidecar size before output.

## Decision

Mantle defines the `function-address-preserves-v1` opaque-evidence profile. Its canonical envelope has role `kamacite-function-address-preserves-receipt`, schema `kamacite.function-address-preserves-receipt.v1`, the BLAKE3 digest of the exact Preserves bytes, and a bounded byte size. Its upstream validation link retains both the Valence JSON artifact digest and Valence's logical `receipt_hash`.

A JSON compatibility projection is optional. When present, it uses role `kamacite-function-address-json-projection`, schema `kamacite.function-address-receipt.v1`, its own artifact-byte digest, and an explicit logical `receipt_hash` equal to the canonical Preserves BLAKE3 identity.

`mantle release function-address-bind --from-preserves-binding` selects the unique typed profile from the verified manifest. The shell reopens each selected file without following symlinks, enforces fixed byte limits, rehashes the bytes against the verified manifest, and parses only the bounded public JSON identity envelopes. It never parses canonical Preserves bytes. The pure core then emits the existing `mantle.function-address-binding.v1` receipt, mapping the canonical Preserves identity to the sidecar and optional Kamacite link fields.

## Alternatives Considered

### Parse Preserves inside Mantle

Rejected because Mantle owns release binding, not Kamacite value semantics. A parser would enlarge the trust boundary and duplicate upstream interpretation.

### Treat JSON as canonical when it is present

Rejected because compatibility projections can drift and cannot replace the canonical Preserves identity.

### Use the Preserves file digest as the Valence logical identity

Rejected because Valence and canonical Preserves identities occupy distinct domains and must be linked explicitly.

### Add a second manifest collection for Preserves

Rejected because the generic opaque-evidence binding already expresses the required confinement, linkage, policy, and non-claim facts.

## Consequences

- Preserves remains an opaque byte artifact at the Mantle boundary.
- Old generic opaque bindings remain source compatible because size and logical-hash DTO fields are optional; this profile requires them.
- Missing, stale, malformed, oversized, role-drifted, scope-drifted, or overclaiming profile data fails before output.
- JSON interoperability remains possible only as a projection bound to canonical Preserves identity.
- Mantle proves bounded artifact identity and release linkage only, not function semantics, upstream verifier correctness, or release eligibility.
