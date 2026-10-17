# Trellis proof release sidecars

Mantle registers Kamacite Trellis proof evidence as a bounded profile of the generic opaque-evidence sidecar contract. The profile preserves recorded-only evidence and accepts required evidence only when Valence authorizes the exact Kamacite candidate as property evidence.

## Authority snapshot

The current registrations were checked against Kamacite revision `980743455f53f81e610c31be8b8b28bb344f3212`, Valence revision `539d756494c3a008c12e8a6dd6121e5db92f3b34`, and Trellis proof commit `d91188e`. ADR 0021's former Kamacite v1 choice is superseded:

- Kamacite's canonical `kamacite.trellis-proof-evidence-profile.v2` Preserves envelope and `kamacite.trellis-proof-evidence-profile.v2.compat-json` projection preserve distinct typed Source, EvidenceChain proof IR, and Receipt identities, verifier status, assumption/dependency roots, property/requirement IDs, and the exact non-claims `not release eligibility`, `not verifier soundness`, and `not downstream correctness`. Its `formal-proof-candidate` role requires `verifier_passed`; it does not decide downstream promotion.
- Valence accepts `AcceptedFormalProof` only with verifier status `passed`, policy acceptance, `VerificationRole::Property`, typed source/proof IR/verifier-receipt and assumption digests, valid spans, property/requirement IDs, and its scoped non-claim. Generic evidence-chain validation also binds its canonical bytes and root. Mantle does not substitute role strings for either upstream validation.
- Trellis's bounded fenced-attempt decision kernel at `d91188e` verifies finite spec/executable equality over caller-classified event relations, not consumer mapping, verifier soundness, or release eligibility. Its generated proof IR receipt is local deterministic linkage, not a Valence-accepted property receipt.

Mantle consumes Valence's accepted role as opaque upstream authority. It binds the measured validation artifact and logical receipt identity but does not re-run Valence semantics or parse proof payloads.

## Registered profile

`kamacite.trellis-proof-evidence-profile.v2` binds:

- a bounded canonical Preserves artifact with its exact BLAKE3 and byte size;
- a Valence validation artifact digest and distinct logical receipt hash;
- release source-archive and binary identities;
- upstream-profile and Mantle-release policy hashes;
- exact producer and validation role pairs;
- claim scope `trellis-proof-identity-linkage-only`;
- required opaque-payload, reference-only, authority, and Kamacite v2's three exact non-claims;
- optionally, one JSON compatibility projection with its own artifact digest and the canonical Preserves identity.

Only Kamacite's `build_trellis_proof_evidence_envelope` over real source, proof IR, verifier-receipt identities, assumptions/dependencies, property and requirement IDs, and all three exact non-claims can produce the canonical v2 envelope. Valence's `validate_trellis_proof_evidence` must separately return a valid `AcceptedFormalProof` report with `VerificationRole::Property`, passed verifier, accepted policy, typed digests, valid spans, and the exact non-claim `Trellis proof evidence preserves scoped proof identity and linkage only, not downstream correctness or release eligibility`; generic `validate_evidence_chain` must also pass. These synthetic binding contract fixtures do not supply a Valence-accepted property receipt. A prior v1 proof binding is rejected, never interpreted as v2.

The external-evidence artifact labels (`kamacite-trellis-proof-preserves-envelope`, `valence-trellis-proof-evidence-profile`, and `kamacite-trellis-proof-json-projection`) are Mantle binding labels. They identify bundle rows; they do not assert that Valence has emitted an accepted-proof receipt.

Artifact observations keep separate identity domains for the canonical bytes, Valence JSON bytes, Valence's logical receipt hash, JSON projection bytes, and the projection's canonical-envelope identity. Stale values fail deterministically.

## Mode behavior

- `optional` accepts absence. A present `recorded-only` or `formal-proof-candidate` binding validated as Valence `recorded_only` reports `recorded-only` and is never promoted.
- `required` passes only for Kamacite `formal-proof-candidate` paired with Valence `property`, and reports `accepted-formal-proof`.
- `required` fails closed for recorded-only evidence, a candidate that remains `recorded_only`, a `property` role attached to a `recorded-only` producer, or any unknown role pair.
- `required` also fails closed for the prior Kamacite v1 proof profile, including when no artifact observations are supplied; optional mode does not treat a stale v1 binding as absence.

The accepted disposition means only that the bounded release binding points to measured Valence-accepted evidence. It does not make Mantle a proof verifier or establish release eligibility by itself.

## Opaque boundary

`crunch-release-core` validates typed metadata and observations. It does not parse Verus source, proof IR, verifier logs, or Preserves internals. Filesystem reads, bounded file-size enforcement, BLAKE3 hashing, and extraction of public JSON identity fields belong in a shell adapter. The optional JSON file remains a projection and never replaces canonical Preserves identity.

The checked fixtures under `tests/fixtures/trellis-proof-release-sidecars/` are synthetic contract fixtures. They prove Mantle's deterministic role-pair, identity-linkage, and fail-closed logic only. Valence's focused accepted/negative profile tests remain the authority for proof acceptance semantics.
