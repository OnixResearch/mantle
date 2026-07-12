# Trellis proof release sidecars

Mantle registers Kamacite Trellis proof evidence as a bounded profile of the generic opaque-evidence sidecar contract. The current profile supports release linkage for recorded-only evidence. It does not accept Trellis evidence as a release proof.

## Authority snapshot

The profile follows these upstream contracts as inspected on 2026-07-12:

- Kamacite revision `de710a092d351e829abfb288d46124e2db8e5b7f` defines canonical schema `kamacite.trellis-proof-evidence-profile.v1`, canonical Preserves bytes, compatibility projection `kamacite.trellis-proof-evidence-profile.v1.compat-json`, and producer roles `recorded-only` and `formal-proof-candidate`. Kamacite explicitly does not decide downstream promotion.
- Valence revision `7a027529dd4b7057cf52e86dc5b258f2a9541545` defines verification roles `property`, `recorded_only`, `boundary`, and `manual_review`. Its shipped formal-proof chain accepts `property` only from Octet-owned formal-proof receipts and keeps imported Trellis records `recorded_only`.
- Trellis revision `3bf9144b99d65ad0c00776d1d5b81b9c8878c222` describes exported proof artifacts as local facts, manual-review evidence, or reference inputs rather than Mantle release eligibility or downstream certification.

Valence's active `trellis-proof-evidence-profile` change has no completed tasks. In particular, Valence has not shipped its proposed profile validator, accepted-formal-proof outcome, positive accepted fixture, graph export, or stack smoke. Mantle therefore has no authoritative accepted Trellis validation receipt to consume.

## Registered profile

`kamacite.trellis-proof-evidence-profile.v1` binds:

- a bounded canonical Preserves artifact with its exact BLAKE3 and byte size;
- a Valence validation artifact digest and distinct logical receipt hash;
- release source-archive and binary identities;
- upstream-profile and Mantle-release policy hashes;
- exact producer and validation role vocabularies;
- claim scope `trellis-proof-identity-linkage-only`;
- required opaque-payload, reference-only, and authority non-claims;
- optionally, one JSON compatibility projection with its own artifact digest and the canonical Preserves identity.

The external-evidence artifact labels (`kamacite-trellis-proof-preserves-envelope`, `valence-trellis-proof-evidence-profile`, and `kamacite-trellis-proof-json-projection`) are Mantle binding labels. They identify bundle rows; they do not assert that Valence has emitted an accepted-proof receipt.

Artifact observations keep separate identity domains for the canonical bytes, Valence JSON bytes, Valence's logical receipt hash, JSON projection bytes, and the projection's canonical-envelope identity. Stale values fail deterministically.

## Mode behavior

- `optional` accepts absence. A valid present binding reports `recorded-only`, including when Kamacite preserved the producer role `formal-proof-candidate`. The candidate role is not promoted.
- `required` fails closed. The current profile accepts only Valence `recorded_only`, so it emits: `required Trellis proof evidence needs an upstream accepted-validation role; the registered profile supports Valence recorded_only validation only`.

A future passing required mode needs an implemented, tested Valence accepted-proof validator and authoritative receipt/profile vocabulary. That change must add a new or explicitly compatible Mantle profile rather than reinterpreting `recorded_only`.

## Opaque boundary

`crunch-release-core` validates typed metadata and observations. It does not parse Verus source, proof IR, verifier logs, or Preserves internals. Filesystem reads, bounded file-size enforcement, BLAKE3 hashing, and extraction of public JSON identity fields belong in a shell adapter. The optional JSON file remains a projection and never replaces canonical Preserves identity.

The checked fixtures under `tests/fixtures/trellis-proof-release-sidecars/` are synthetic contract fixtures. They prove Mantle's deterministic shape and fail-closed logic only; they are not evidence that the currently missing Valence acceptance path exists.
