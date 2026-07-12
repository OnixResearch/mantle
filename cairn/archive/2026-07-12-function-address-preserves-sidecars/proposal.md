## Why

Mantle should bind function-address evidence to release artifacts without parsing Rust or reinterpreting Valence/Kamacite internals. If Kamacite receipts become canonical Preserves values, Mantle needs a release-evidence sidecar contract that treats Preserves receipts as opaque, hash-bound evidence objects.

## What Changes

- Accept canonical Kamacite Preserves function-address receipt metadata in release evidence bindings.
- Bind Preserves receipt hash, Valence receipt hash, source archive digest, release binary digest, schema version, role, claim scope, and non-claims.
- Keep JSON sidecars as compatibility projections only when they bind to canonical Preserves identity.
- Add positive and negative fixtures for opaque Preserves sidecar binding.

## Impact

Mantle can bind durable function-address receipt identity into releases without claiming semantic correctness or taking ownership of the Preserves schema.
