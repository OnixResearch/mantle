# ADR 0119: Keep builder protocols at the Nix adapter edge

- **Status:** Proposed
- **Date:** 2026-09-04

## Context

Nix is developing builder-facing store APIs. The merged experimental `builder-rpc-v0` feature reuses a limited daemon protocol. Draft PR 13768 proposes a separate Varlink service with object upload, derivation admission, and output submission operations.

Mantle already owns corresponding build and store capabilities. It has its own castore, PathInfo state, scheduler, dynamic-plan format, execution policy, and output admission.

A literal adoption of Nix operation names or protocol types would make Mantle's core depend on an experimental Nix interface. It would also weaken ADR 0011, which selects native dynamic plans as Mantle's core interface.

Mantle still needs explicit compatibility behavior. Imported Nix derivations can require builder services that Mantle does not provide, and generated compatibility derivations can request effects beyond their producer.

## Decision

Mantle will keep builder protocol parsing and transport inside declared Nix adapters.

Mantle core contracts will use Mantle-owned capabilities for:

- immutable object admission;
- bounded execution-unit admission;
- declared-result binding;
- dynamic child-authority attenuation.

A Nix adapter may map supported protocol operations onto these capabilities. Protocol operation names, Varlink values, Nix daemon message types, file descriptors, and builder-RPC versions will not enter core commands, ports, decisions, receipts, or native plans.

Dynamic children will pass one Mantle-owned attenuation decision before registration. A generated child cannot receive broader effect authority than its producer.

Until a consumer-driven change implements a supported adapter, `builder-rpc-v0` remains an explicit Nix compatibility blocker. Unknown mandatory Nix system features also fail closed at the adapter edge.

## Consequences

- Native dynamic plans remain usable without Nix.
- Nix compatibility remains replaceable and version-scoped.
- Mantle can learn from upstream builder API requirements without adopting its protocol as a domain model.
- A future adapter needs translation, negotiation, bounds, positive tests, and negative tests.
- Some Nix derivations will fail before planning because they require unsupported builder services.
- The existing client-facing Nix remote-service gateway does not grant builder-sandbox authority.

## Alternatives Considered

### Adopt the Nix builder protocol as Mantle's native API

Rejected because the protocol is experimental and Nix-specific. This choice would couple native plans and core types to upstream daemon evolution.

### Ignore required Nix system features

Rejected because Mantle could execute a derivation under different builder semantics. Missing output bindings or store-service behavior would then appear as ordinary build failures.

### Add a transport-neutral builder service now

Rejected because no current Mantle consumer needs the complete service. Dynamic child attenuation is the current requirement and has a smaller authority surface.

### Remove all Nix compatibility

Rejected because concrete Nix derivation, store, cache, and remote-client compatibility remain useful adapter surfaces. They do not need to define Mantle's core.

## References

- [ADR 0011](0011-native-dynamic-plans.md)
- [ADR 0077](0077-adopt-nix-derivation-at-the-nix-compatibility-boundary.md)
- [ADR 0116](0116-adapt-nix-remote-clients-without-transferring-authority.md)
- <https://github.com/NixOS/nix/pull/13768>
- <https://github.com/NixOS/nix/pull/13768#issuecomment-5540182998>
- <https://github.com/NixOS/nix/pull/15793>
