# Change: Attenuate dynamic child authority

## Why

Mantle admits native `mantle-plan-v1` units and traditional generated `.drv` files after a producer build. Both paths currently register children with `ExecutionProfile::native_compatibility()` instead of an authority derived from the producer.

Mantle also grants network access to a `builtin:fetchurl` fixed-output action from the child shape. This decision occurs without a parent authority ceiling. An offline producer can therefore cause a generated child to receive network authority.

Current Nix derivation import does not classify required builder-facing system features. A derivation that requires `builder-rpc-v0` can reach ordinary Mantle planning even though Mantle does not implement that protocol.

These gaps must close without making Nix daemon operations, Varlink, or builder-RPC types part of Mantle's core API.

## What Changes

- Add a Mantle-owned pure attenuation decision for dynamic child authority.
- Derive each parent ceiling from its admitted execution profile and effective action policy.
- Derive each child request from its execution profile, action kind, dynamic policy, and requested effects.
- Require native plans and traditional `.drv` compatibility children to pass the same attenuation decision before registry or scheduler mutation.
- Replace dynamic registration defaults with an explicit admitted child profile and authority-decision identity.
- Classify Nix required system features at the Nix producer edge.
- Reject `builder-rpc-v0`, malformed requirements, and unknown mandatory builder features before successful graph publication.
- Keep Nix protocol names and transport types inside Nix adapters and adapter fixtures.
- Add deterministic positive, negative, mutation, and architecture tests.

## Capability Outcome

- **Immediate outcome:** An offline producer cannot create a network-capable dynamic child.
- **Durable capability:** Mantle can admit child work only through monotonic authority attenuation.
- **Current consumers:** Native dynamic plans and traditional generated `.drv` compatibility discovery.
- **Adoption path:** Route both existing Worker lanes through one pure decision before registry insertion.
- **Maintenance owner:** `crunch-build` owns attenuation. The Nix producer adapter owns Nix feature classification.
- **Repeatability evidence:** Focused core tests, Worker integration tests, Nix fixture tests, and a source architecture guard.

## Dependencies

- ADR 0011 defines native dynamic plans as Mantle's core dynamic interface.
- ADR 0077 confines Nix derivation parsing to the Nix compatibility boundary.
- ADR 0106 requires explicit fallible build-boundary admission.
- Accepted application-architecture requirements define core and adapter dependency direction.

## Non-Goals

- Implementing Nix `builder-rpc-v0`.
- Implementing the draft Varlink service from Nix PR 13768.
- Adopting `AddToStore`, `AddDerivation`, or `SubmitOutput` as Mantle domain APIs.
- Replacing Mantle's native dynamic-plan format with Nix derivations.
- Claiming arbitrary Nix derivation or daemon compatibility.

## Impact

- **Affected specs:** `dynamic-derivation-admission`, `foreign-derivation-import`, and `build-tool-boundary`.
- **Affected code:** `crunch-build` dynamic admission, Worker registration, execution profiles, network policy projection, and Nix producer adapters.
- **Affected documentation:** ADR 0119 and the dynamic-derivation and Nix compatibility guides.
- **Compatibility:** Previously accepted dynamic children remain accepted only when their effective authority does not exceed the producer ceiling.
- **Intentional behavior change:** A generated fixed-output fetcher from an offline parent fails before network or scheduler effects.
