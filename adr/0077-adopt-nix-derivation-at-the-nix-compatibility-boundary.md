# ADR 0077: Adopt `nix-derivation` at the Nix compatibility boundary

## Status

Accepted

## Context

Mantle uses an adapted `nix-compat` for native derivations, store protocols, and castore integration. Native derivations use BLAKE3 and configurable logical store prefixes.

Concrete Nix `.drv` files use Nix identity rules. Those rules require canonical Nix ATerm bytes, SHA-256 derivation identities, and the standard `/nix/store` prefix.

The `nix-derivation` crate implements the Nix 2.34 derivation metadata boundary. It does not evaluate Nix, execute builders, or implement a store.

## Decision Drivers

- Keep Nix and Mantle hash domains separate.
- Use one reviewed parser for concrete Nix `.drv` input.
- Preserve existing Mantle graph and receipt schemas.
- Keep byte, collection, field, closure, and depth limits under Mantle authority.
- Reject unsupported output and dynamic-input forms before graph publication.
- Preserve the adapted `nix-compat` implementation for native Mantle behavior.

## Decision

Mantle will use `nix-derivation` version `0.1.0` only through `src/nix_derivation_adapter.rs`.

The admitted package has crates.io SHA-256 `a5d03dfde06a8ce7e0e007f4795ab74c546e5c7d6c6375a08ceb20677ec8a074`. Its source records upstream commit `2cfc0f90ed83ea3cc983e5c305f89494a6df073e` and license `Apache-2.0`.

The adapter receives owned bytes, one logical `/nix/store/*.drv` identity, and explicit limits. It derives the out-of-band name from that logical identity.

The adapter returns Mantle-owned projection types. No public plan, receipt, graph, store, or CLI type exposes an upstream crate type.

Direct Nix `.drv` inputs and backend-produced Nix closures use this adapter. Guix prefix-rewrite inputs remain on the prior parser until a separate parity gate passes.

Nix ATerm, derivation, output, and store-path identity remains SHA-256-compatible. Mantle graph, policy, plan, receipt, and evidence identity remains BLAKE3.

Floating, deferred, impure, Git-addressed, text-addressed, and recursive dynamic forms remain explicit unsupported cases for the current foreign graph. The adapter does not flatten them.

## Alternatives Considered

### Replace adapted `nix-compat` everywhere

Rejected because this would remove Mantle-native BLAKE3 and configurable-prefix behavior. It would also expand the dependency beyond its reviewed scope.

### Keep `nix-derivation` as a test-only oracle

Rejected for concrete `/nix/store` admission because the old production parser carries Mantle-specific changes. A test-only oracle would not close the production identity boundary.

### Copy the upstream implementation into Mantle

Rejected because a copied fork would add maintenance and review work without adding Mantle authority.

### Cut over Guix prefix rewriting

Deferred because `nix-derivation` intentionally targets `/nix/store`. Prefix rewriting needs separate evidence and policy.

## Consequences

- One exact external crate enters the root package dependency graph.
- A deterministic guard checks package source parity and direct-import confinement.
- Existing foreign graph, package-index, receipt, and CLI schemas remain unchanged.
- Existing synthetic fixed-output fixtures use Nix-derived output paths.
- Unsupported Nix forms fail before graph publication.
- Passing checks does not prove evaluator parity, build success, output correctness, store trust, or release eligibility.

## Rollback

Rollback removes `nix-derivation` and restores the pre-adoption parser call sites from revision `465e4b45abd7ff8887ada01df93c9a45d30faf06`.

The dependency and adapter must roll back together. Public foreign artifact schemas do not change.

## References

- <https://github.com/cachix/nix-derivation>
- [ADR 0003](0003-configurable-store-prefix.md)
- [ADR 0046](0046-realize-foreign-graphs-through-a-receipt-bound-adapter.md)
- [ADR 0054](0054-select-snix-backports-by-mantle-compatibility-boundary.md)
