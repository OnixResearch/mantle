# Trellis proof release sidecars

Mantle registers Kamacite Trellis proof evidence as a bounded profile of the generic opaque-evidence sidecar contract. The profile preserves recorded-only evidence and accepts required evidence only when Valence authorizes the exact Kamacite candidate as property evidence.

## Authority snapshot

The profile follows these upstream contracts as inspected on 2026-07-14:

- Kamacite revision `de710a092d351e829abfb288d46124e2db8e5b7f` defines canonical schema `kamacite.trellis-proof-evidence-profile.v1`, canonical Preserves bytes, compatibility projection `kamacite.trellis-proof-evidence-profile.v1.compat-json`, and producer roles `recorded-only` and `formal-proof-candidate`. Kamacite explicitly does not decide downstream promotion.
- Valence commit `27b8b212` archives `trellis-proof-evidence-profile`. Its validator distinguishes `reference_only`, `formal_proof_candidate`, and `accepted_formal_proof`, and accepts the last only with verifier status `passed`, policy acceptance, verification role `property`, required assumptions, typed identities, spans, and non-claims.
- Trellis revision `3bf9144b99d65ad0c00776d1d5b81b9c8878c222` describes exported proof artifacts as local facts, manual-review evidence, or reference inputs rather than Mantle release eligibility or downstream certification.

Mantle consumes Valence's accepted role as opaque upstream authority. It binds the measured validation artifact and logical receipt identity but does not re-run Valence semantics or parse proof payloads.

## Registered profile

`kamacite.trellis-proof-evidence-profile.v1` binds:

- a bounded canonical Preserves artifact with its exact BLAKE3 and byte size;
- a Valence validation artifact digest and distinct logical receipt hash;
- release source-archive and binary identities;
- upstream-profile and Mantle-release policy hashes;
- exact producer and validation role pairs;
- claim scope `trellis-proof-identity-linkage-only`;
- required opaque-payload, reference-only, and authority non-claims;
- optionally, one JSON compatibility projection with its own artifact digest and the canonical Preserves identity.

The external-evidence artifact labels (`kamacite-trellis-proof-preserves-envelope`, `valence-trellis-proof-evidence-profile`, and `kamacite-trellis-proof-json-projection`) are Mantle binding labels. They identify bundle rows; they do not assert that Valence has emitted an accepted-proof receipt.

Artifact observations keep separate identity domains for the canonical bytes, Valence JSON bytes, Valence's logical receipt hash, JSON projection bytes, and the projection's canonical-envelope identity. Stale values fail deterministically.

## Mode behavior

- `optional` accepts absence. A present `recorded-only` or `formal-proof-candidate` binding validated as Valence `recorded_only` reports `recorded-only` and is never promoted.
- `required` passes only for Kamacite `formal-proof-candidate` paired with Valence `property`, and reports `accepted-formal-proof`.
- `required` fails closed for recorded-only evidence, a candidate that remains `recorded_only`, a `property` role attached to a `recorded-only` producer, or any unknown pair.

The accepted disposition means only that the bounded release binding points to measured Valence-accepted evidence. It does not make Mantle a proof verifier or establish release eligibility by itself.

## Opaque boundary

`crunch-release-core` validates typed metadata and observations. It does not parse Verus source, proof IR, verifier logs, or Preserves internals. Filesystem reads, bounded file-size enforcement, BLAKE3 hashing, and extraction of public JSON identity fields belong in a shell adapter. The optional JSON file remains a projection and never replaces canonical Preserves identity.

The checked fixtures under `tests/fixtures/trellis-proof-release-sidecars/` are synthetic contract fixtures. They prove Mantle's deterministic role-pair, identity-linkage, and fail-closed logic only. Valence's focused accepted/negative profile tests remain the authority for proof acceptance semantics.

## Fenced-attempt evidence

The remote-admission profile uses this accepted v1 path. Its exact artifacts are under `evidence/trellis/remote-admission-v1/`.

The profile binds Trellis revision `8de4b24aa2d66cc2e6ec966d686df023492265d3`. It also binds the executable oracle and Mantle projection source.

Read [Trellis remote-admission evidence](trellis-remote-admission.md) for the mapping, unsupported cases, validation commands, and claim boundary.

This evidence does not satisfy a release requirement by itself. A release bundle must still bind its own source and binary identities.
