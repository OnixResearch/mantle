# Architecture review: Mantlepkgs package domains

## Question

Which Ekala package-collection patterns fit Mantle without importing Nix package authority or deployment scope?

## Inspected evidence

- Mantle ADR 0010 keeps Mantle build-shaped.
- Mantle ADR 0056 selects concrete-graph generation and a no-Nix consumer.
- `cairn/changes/generate-mantlepkgs-from-nixpkgs/` defines the active catalog producer boundary.
- Ekala `corepkgs` separates a reviewed core from ecosystem package repositories.
- Ekala `corepkgs` uses named variants and selected separate test derivations.
- Ekala `repo-packages.nix` demonstrates deterministic filesystem package discovery and public package enumeration.
- The `corepkgs` repository reported an MIT license during this planning review.

External sources:

- <https://github.com/ekala-project/corepkgs>
- <https://github.com/ekala-project/corepkgs/blob/master/docs/major-differences-nixpkgs.md>
- <https://github.com/ekala-project/corepkgs/blob/master/repo-packages.nix>

## Decision

Adapt package domains, explicit variants, deterministic composition, and separate validation roots. Use one pinned `corepkgs` revision as an external corpus after a recorded license and provenance review.

Do not copy Nix expressions into Mantle product code. Do not add overlay precedence or package trust based on a domain label.

## Owner

Mantle owns catalog structure, generated identities, validation-root planning, and evidence. The foreign producer owns only bounded evaluation observations.

## Next action

Implement the typed domain contracts and pure composition core after `generate-mantlepkgs-from-nixpkgs` establishes its versioned catalog format.

## Non-claim

This review is planning evidence. No Ekala code was built, copied, or admitted. No package compatibility or correctness claim follows.
