# Architecture review: Mantlepkgs source updates

## Question

Which Ekala update patterns fit Mantle's typed policy, functional core, and evidence boundaries?

## Inspected evidence

- Mantle project refresh already separates declared inputs from live resolution adapters.
- Mantle ADR 0056 requires locked producer inputs and visible blockers.
- Ekala `corepkgs` provides common updater combinators and version-source adapters.
- `ekapkgs-update` provides source discovery, version policy, source rewriting, OSV observations, Repology observations, and review output.
- The inspected `ekapkgs-update` metadata path invokes Nix repeatedly.
- The inspected vulnerability path converts selected fetch failures into empty result sets.
- The `ekapkgs-update` repository reported a BSD-3-Clause license during this planning review.

External sources:

- <https://github.com/ekala-project/corepkgs/tree/master/common-updater>
- <https://github.com/ekala-project/ekapkgs-update>
- <https://github.com/ekala-project/ekapkgs-update/blob/master/ekapkgs-update/src/cve/mod.rs>

## Decision

Adapt typed source adapters, version policy, conservative update planning, OSV observations, and Repology observations.

Reimplement the behavior with Nickel contracts, a Rust functional core, explicit unavailable states, and preimage-bound mutation. Do not adopt Nix subprocess orchestration or failure-to-empty behavior.

## Owner

Mantle owns policy normalization, candidate selection, mutation planning, validation linkage, advisory status, and receipts. Shell adapters own bounded network and filesystem effects.

## Next action

Implement saved-observation replay and pure candidate selection after catalog domains and report contracts are stable.

## Non-claim

This review is planning evidence. It does not establish advisory completeness, source trust, package correctness, or upstream implementation quality.
