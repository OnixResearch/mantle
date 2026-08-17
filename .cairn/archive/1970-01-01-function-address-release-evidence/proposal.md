## Why

Octet, Kamacite, and Valence can make function-address evidence meaningful, but a release consumer still needs to know which built artifact shipped with which evidence. Mantle owns that build/release boundary: artifact paths, sidecars, release bundles, external evidence metadata, and bundle-local digest verification.

Function-address evidence should therefore enter Mantle as opaque external evidence that has already been validated by Valence and/or encoded by Kamacite. Mantle should bind that evidence to the release binary and source archive without parsing Rust, recomputing function addresses, or claiming semantic correctness.

## What Changes

- Add function-address evidence as an optional or required release profile input, reusing Mantle's external evidence and sidecar binding model.
- Bind function-address sidecar bytes, Valence verification/graph-report receipt bytes, Kamacite receipt identity when present, release binary identity, source archive identity, roles, schemas, claim scope, and non-claims.
- Fail required release verification on missing sidecars, stale BLAKE3 digests, wrong roles/schemas, unsupported claim scopes, binary/source identity mismatch, weakened non-claims, or overclaims.
- Add positive and negative release fixture coverage while keeping the evidence opaque to Mantle semantics.
- Document that Mantle proves bundle-local evidence binding only; Octet, Kamacite, Valence, and Cairn own their layers.

## Impact

- **Files**: Mantle release-provenance validation, release evidence metadata, fixtures, docs, CLI examples if needed, and Cairn lifecycle artifacts.
- **Testing**: positive/negative release evidence fixtures, focused release verification tests, constants/profile drift checks, Cairn validation/gates, and package/release checks before archive.
