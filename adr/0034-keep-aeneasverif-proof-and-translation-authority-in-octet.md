# ADR 0034: Keep AeneasVerif proof and translation authority in Octet

## Status

Accepted (2026-07-26)

## Context

Octet has a pinned Aeneas, Charon, and Lean provider. It owns bounded execution, theorem admission, proof receipts, and artifact validation.

Valence validates the resulting evidence identities, links, roles, and non-claims. Trellis owns selected theorem campaigns over its exact runtime source.

Mantle can build pinned tool closures and generated artifacts. That capability does not give Mantle authority over verifier policy or translation semantics.

Future Eurydice and Scylla work creates the same boundary question. Eurydice produces C from Rust, while Scylla experimentally produces Rust from highly regular C.

## Decision

Mantle will keep AeneasVerif proof and translation authority outside Mantle core.

Octet owns these surfaces:

- Charon, Aeneas, Lean, Eurydice, and Scylla execution profiles
- source and selected-item admission
- theorem and migration policy
- supported, blocked, and failed outcome classes
- translation and proof receipts
- claim and non-claim boundaries

Mantle can realize an Octet-authored derivation that contains pinned tools. Mantle can also build or package generated Lean, C, Rust, and binary artifacts.

A Mantle build report records the selected derivation, inputs, outputs, sandbox observations, and attached external evidence. It does not reinterpret an Octet result.

For Eurydice, Mantle can compile and package generated C after Octet emits a separate generation receipt. The receipt does not transfer a Rust theorem to C.

For Scylla, Mantle can build generated Rust after Octet records the migration inputs and output. The build does not prove C-to-Rust equivalence.

Mantle release policy can require an exact Octet receipt and accepted Valence report. It must carry them as external evidence with their existing roles and non-claims.

Mantle must not add Aeneas-specific theorem admission, Charon or LLBC semantics, Eurydice guarantees, Scylla migration semantics, or proof-role promotion.

## Alternatives Considered

### Make Mantle own the complete verification toolchain

Rejected. This mixes build observations with verifier and theorem authority.

### Teach Mantle to validate LLBC and generated proof models

Rejected. Valence owns evidence linkage, while Octet owns compiler and verifier execution policy.

### Treat successful generated-artifact builds as translated-program proof

Rejected. A successful build does not prove compiler correctness or semantic equivalence.

## Consequences

- Mantle can package AeneasVerif tools without becoming their policy owner.
- Octet receipts remain the source of proof and translation observations.
- Valence reports remain the source of canonical evidence linkage.
- Mantle release bundles preserve external roles and non-claims.
- Future integration needs an Octet producer contract before Mantle adds a product-facing example or package.
