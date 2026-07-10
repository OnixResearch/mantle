## Why

Trellis proof evidence can support release review only if Mantle binds it to the release source and binary artifacts as opaque evidence. Mantle should not parse Verus source or proof IR; it should verify that canonical Trellis proof evidence and Valence validation receipts are present, linked, and scoped to release-local claims.

## What Changes

- Add Trellis proof evidence as a supported profile of Mantle's generic opaque evidence sidecar binding.
- Bind canonical Kamacite envelope hash, Valence proof evidence validation hash, source archive hash, release binary hash, proof-scope metadata, policy hashes, claim scope, and non-claims.
- Preserve reference-only versus accepted formal-proof role metadata without deciding theorem truth.
- Add positive and negative release-evidence fixtures.

## Impact

Mantle can include Trellis verified-logic evidence in release bundles while keeping proof payloads opaque and avoiding semantic correctness or release-eligibility overclaims.
