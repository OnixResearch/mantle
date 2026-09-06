## Why

The standalone build interchange package pins BLAKE3 1.8.2 even though it uses
only stable inherent hashing APIs. This prevents composition with an admitted
consumer that requires BLAKE3 1.8.5 or later. The store's exact 1.8.2 pin has a
different purpose: its `traits-preview` feature must match digest 0.10.

## What Changes

- Permit compatible BLAKE3 1.x selection in `mantle-build-contract` only.
- Keep the store pins, selected owner lock, features, wire schemas, identities,
  admission policy, and authority unchanged.
- Maintain independent consumer checks locked to BLAKE3 1.8.2 and 1.8.7.
- Check original producer fixtures and rejection controls in both consumers.
- Publish scoped evidence and check the actual Neural Stream dependency graph.

## Impact

The owner is Mantle. The changed library is `no_std + alloc`, without host
capabilities. Nix and Cargo remain imperative verification adapters. No source
algorithm is copied or replaced. The native CLI package, trainer, store trust,
and downstream production admission remain outside this change's claims.
