# Generate Mantlepkgs from locked Nixpkgs graphs

## Why

Mantle can import and realize one concrete Nixpkgs derivation graph. It does not yet generate a reusable Mantle package collection from many selected Nixpkgs packages.

Manual package export does not scale. It also does not provide one typed catalog, shared dependency reuse, package blockers, or repeatable update evidence.

## What Changes

- Add a typed Nickel manifest for a locked Nixpkgs source, target systems, package selectors, conversion policy, source policy, and named limits.
- Add a producer command that uses Nix only before artifact publication.
- Compile selected concrete graphs into one deterministic Mantlepkgs catalog with shared dependencies.
- Recompute target derivations and outputs under the configured Mantle store prefix with BLAKE3 identities.
- Build catalog packages through Mantle without Nix in the consumer environment.
- Retain unsupported packages and exact blockers instead of dropping them or using Nix as a fallback.
- Keep Nario v2 as an optional input transport. The core converter must also work with existing source bundles.

## Impact

- **Planned files**: a new pure Mantlepkgs core crate, Mantle CLI adapters, typed Nickel contracts, generated catalog artifacts, fixtures, operator documentation, and receipt schemas.
- **Testing**: deterministic catalog tests, positive source rebuilds, negative package fixtures, no-Nix consumer tests, machine-contract checks, and Cairn gates.
- **Compatibility**: this path does not preserve original `/nix/store` output identity or promise reuse from Nix binary caches.
- **Current effect**: lifecycle planning only. This change does not yet convert or rebuild packages.
