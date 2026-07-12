## Why

Cairn currently consumes a wrapper JSON for Mantle function-address readiness input. Mantle should own a stable release binding receipt schema so Cairn does not rely on ad hoc field names or custom smoke adapters.

## What Changes

- Define a versioned Mantle function-address release binding receipt schema.
- Include required hashes, roles, schemas, claim scope, source/binary linkage, optional Kamacite metadata, and non-claim boundary.
- Add positive and negative schema fixtures.
- Document the mapping from `FunctionAddressReleaseVerification` to Cairn readiness input.

## Impact

Mantle and Cairn get a stable contract for function-address release evidence binding without making Mantle parse Valence internals or Rust source.
