# Plan Mantlepkgs source updates

## Why

Mantlepkgs binds locked package graphs, but it has no package-update policy or reviewable update plan. Manual lock edits can hide version-policy decisions, incomplete advisory data, and unsafe source rewrites.

Ekala updater projects provide useful source and version-policy patterns. Their Nix subprocess and failure behavior do not fit Mantle's evidence or functional-core boundaries.

## What Changes

- Add typed Nickel update policies with named limits.
- Add bounded source-observation adapters for declared source kinds.
- Select update candidates through a pure deterministic core.
- Emit preimage-bound dry-run mutations before any write.
- Record OSV and Repology observations with explicit success, unavailable, and failed states.
- Bind package builds, separate validation roots, and impact reports to candidate evidence.
- Keep network, file mutation, and producer execution in a thin shell.

## Non-Goals

- Treating OSV or Repology as package trust or release authority.
- Treating a network failure as zero advisories.
- Running arbitrary package-provided update scripts.
- Applying text substitutions without exact preimage and structure checks.
- Changing source locks without an explicit execute action.

## Dependencies

- `generate-mantlepkgs-from-nixpkgs` supplies locked catalog generations.
- `structure-mantlepkgs-domains` supplies variants and separate validation roots.
- `report-mantlepkgs-impact` supplies candidate impact evidence.

## Impact

- **Affected specs:** new `mantlepkgs-update-plans` specification.
- **Affected code:** Nickel policy contracts, pure update core, source adapters, advisory adapters, mutation shell, and CLI reports.
- **Affected evidence:** source observations, candidate decisions, advisory status, preimage-bound mutations, validation results, and impact links.
- **Compatibility:** mutation and report schemas are versioned. Existing project refresh behavior remains separate.
