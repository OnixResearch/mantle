## Why

Mantle release bundles should be able to carry `.preserves` evidence artifacts from Molten/Aspen, Cairn, Valence, and other producers without making Mantle depend on their runtime semantics. A release role contract lets Mantle verify bundle-local linkage while preserving format-specific identity.

## What Changes

- Add Preserves release evidence carrier roles.
- Validate canonical digest, schema label, producer, role, claim scope, bundle path, and non-claims.
- Keep Preserves payload interpretation opaque unless a Mantle release profile declares a specific adapter.
- Add positive and negative fixtures for canonical carriers, stale digests, wrong schemas, wrong roles, missing non-claims, and semantic overclaims.

## Impact

- Release bundles can carry Preserves artifacts first-class.
- Mantle stays generic and validates only the release slice it consumes.
