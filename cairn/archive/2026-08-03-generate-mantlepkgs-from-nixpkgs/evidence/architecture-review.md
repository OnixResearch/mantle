# Mantlepkgs architecture review

## Goal

Define a package-set converter that rebuilds locked Nixpkgs selections under Mantle identities without Nix during consumption.

## Considered approaches

### Direct Nix source translation

Rejected. General Nix source uses evaluator behavior that a bounded Nickel rewrite cannot preserve.

### Concrete derivation graph conversion

Selected. The existing foreign import boundary already preserves the build facts required by Mantle.

### Exact Nix cache path preservation

Rejected for Mantlepkgs. It supports compatibility import, but it does not create Mantle-owned target identity or prove rebuilding.

### Handwritten Nickel output for every imported derivation

Rejected as the generated interchange. A typed Nickel catalog can reference canonical machine graph artifacts without pretending they are maintainable source ports.

### Nario as the package recipe

Rejected. The reviewed Determinate Nix 3.12 announcement defines Nario as NAR payloads plus path metadata.

Nario remains useful as an optional payload transport. It does not replace derivation export, package selection, or producer receipts.

## Adversarial findings

- Every selected package needs an explicit success or blocker disposition.
- Shared foreign nodes must fail on conflicting facts instead of using first-wins merging.
- Clean rebuild evidence must disable substitution and require built outcomes.
- Source transport identity must not change package recipe identity.
- Hidden `/nix/store` assumptions can remain in source payloads and need explicit blockers or package repairs.
- Initial cohort success cannot establish full Nixpkgs support.

## External evidence

- Determinate Systems, “Changelog: introducing `nix nario`,” published 2025-10-31: <https://determinate.systems/blog/changelog-determinate-nix-3120/>
- Mantle accepted foreign import requirements: `cairn/specs/foreign-derivation-import/spec.md`.
- Mantle accepted archive compatibility boundary: `cairn/specs/store-transports/spec.md`.

## Validation state

This file records planning evidence only. Implementation, package rebuilds, and format compatibility remain unproven.
