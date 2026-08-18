## Why

Mantle should not add a separate release-binding shape for every evidence kind. Function-address Preserves sidecars show the right pattern: Mantle binds opaque, typed, hash-chained evidence objects to source and binary artifacts without interpreting their internals.

## What Changes

- Define a generic opaque evidence sidecar binding contract for release provenance.
- Bind canonical evidence envelope hash, evidence kind, profile version, Valence validation hash, source artifact hash, release binary hash, policy hashes, claim scope, and non-claims.
- Treat function-address as the first concrete sidecar profile using the generic binding.
- Keep evidence payload parsing outside Mantle release validation core.

## Impact

Mantle gains one reusable release-evidence sidecar model for function-address, proof, lint, dependency, build, attestation, and lifecycle evidence profiles.
