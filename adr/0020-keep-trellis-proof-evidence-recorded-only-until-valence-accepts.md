# ADR 0020: Keep Trellis proof evidence recorded-only until Valence accepts it

## Status

Accepted

## Context

Kamacite can canonicalize Trellis proof evidence into Preserves bytes and preserve producer roles `recorded-only` and `formal-proof-candidate`. Kamacite explicitly does not decide downstream proof promotion. Trellis likewise limits exported proof artifacts to local facts, manual review, or reference inputs rather than Mantle release eligibility.

Valence owns evidence semantics. Its shipped role vocabulary includes `property` and `recorded_only`, but its shipped formal-proof-chain validator accepts property evidence only through Octet-owned receipts and keeps Trellis imports recorded-only. Valence's active `trellis-proof-evidence-profile` change proposes an accepted-formal-proof outcome, but its implementation, accepted fixture, and stack-smoke tasks are all incomplete.

Mantle needs to bind existing Trellis artifacts to release source and binary identities without inventing a Valence accepted receipt or turning Kamacite's candidate role into acceptance.

## Decision Drivers

- Preserve Kamacite's authoritative canonical Preserves identity and exact producer roles.
- Preserve Valence's current `recorded_only` counting boundary.
- Detect stale artifact bytes, logical receipt identity, release links, policy links, projection links, and weakened non-claims.
- Keep Verus source, proof IR, verifier logs, and Preserves internals outside Mantle's pure core.
- Fail closed when policy requires accepted proof but no upstream acceptance authority exists.
- Make future promotion an explicit profile evolution rather than a reinterpretation of existing evidence.

## Decision

Mantle registers `kamacite.trellis-proof-evidence-profile.v1` under its generic opaque-evidence sidecar binding. The profile accepts Kamacite producer role `recorded-only` or `formal-proof-candidate` only when the Valence validation role remains exactly `recorded_only`.

Optional mode may report a valid present binding only as `recorded-only`; it preserves the original Kamacite producer role separately. Required mode cannot pass under this profile and returns an explicit missing-acceptance-authority diagnostic.

The profile binds canonical Preserves artifact size and BLAKE3, Valence artifact BLAKE3 and distinct logical receipt hash, release source and binary identities, policy hashes, exact claim scope, and mandatory non-claims. One optional JSON projection may be linked by its own artifact digest and canonical-envelope identity, but it remains non-authoritative.

The pure core receives bounded artifact observations and compares them to the typed manifest. A shell adapter may read and hash files or extract public JSON identity fields; the core never parses proof payloads or Preserves internals.

## Alternatives Considered

### Treat `formal-proof-candidate` as accepted proof

Rejected because Kamacite explicitly leaves promotion to downstream policy and Valence has not implemented the proposed accepted Trellis validator.

### Reuse Valence `property` for Trellis evidence now

Rejected because current Valence validation reserves property counting for Octet-owned formal-proof receipts and rejects promoted Trellis references.

### Omit required mode until Valence is ready

Rejected because a deterministic fail-closed required result makes the unavailable authority explicit and prevents callers from treating recorded-only evidence as sufficient.

### Parse Trellis or Preserves payloads in Mantle

Rejected because payload semantics belong to Trellis, Kamacite, and Valence. Mantle owns bounded release identity and linkage.

## Consequences

- Mantle can package and validate recorded-only Trellis linkage without claiming proof acceptance.
- A Kamacite formal-proof candidate remains visible but counts only as recorded-only in Mantle.
- Accepted-proof positive validation and end-to-end stack smoke remain blocked on Valence's unfinished profile.
- Future accepted support requires authoritative Valence role/schema/receipt semantics plus positive and adversarial upstream evidence, followed by explicit Mantle profile evolution.
- JSON interoperability remains optional and subordinate to canonical Preserves identity.
