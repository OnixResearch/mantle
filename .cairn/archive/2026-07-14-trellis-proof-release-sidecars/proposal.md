## Why

Trellis proof evidence can support release review only if Mantle binds it to the release source and binary artifacts as opaque evidence. Mantle should not parse Verus source or proof IR; it should verify that canonical Trellis proof evidence and Valence validation receipts are present, linked, and scoped to release-local claims.

## What Changes

- Add Trellis proof evidence as a supported profile of Mantle's generic opaque evidence sidecar binding.
- Bind canonical Kamacite envelope hash, Valence proof evidence validation hash, source archive hash, release binary hash, proof-scope metadata, policy hashes, claim scope, and non-claims.
- Preserve authoritative producer and validation role metadata without deciding theorem truth or manufacturing accepted formal-proof authority.
- Add positive recorded-only and adversarial negative release-evidence fixtures; defer accepted-proof evidence until Valence ships the required validator and receipt.

## Impact

Mantle can include Trellis verified-logic evidence in release bundles while keeping proof payloads opaque and avoiding semantic correctness or release-eligibility overclaims.
