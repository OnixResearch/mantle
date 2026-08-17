# Current Blocker — Forge-agnostic VCS project inputs

Date: 2026-07-01

## Question

Can `forge-agnostic-vcs-inputs` be honestly drained from the current tree?

## Inspected evidence

- Darcs, Pijul, and Fossil manifest schemas now exist with explicit repository URLs and VCS-native selectors.
- Lock entries now record VCS-native identity metadata plus source-tree content digests: Darcs context/weak-hash, Pijul state/change, and Fossil check-in.
- Generated `.mantle/inputs.ncl` data, static soundness, retention digests, project attestations, fetch-policy source identity, and project display all handle the new VCS kinds.
- Refresh adapters now expose resolver hooks for non-Git VCS identity and tree hashing. The live resolver has best-effort Darcs/Pijul/Fossil materializers and deterministic `unsupported-VCS-tool` diagnostics for missing tools or unsupported subfeatures such as Pijul change selectors without state proof.
- Tests cover positive selector/lock/generated/adapter paths and negative forge shorthand, missing identity, ambiguous selector, and missing-tool blocker paths.

## Decision

Unblocked. The prior blocker has been resolved by the implementation and validation evidence in this change. Unsupported VCS subfeatures now fail closed instead of being reported as successful source support.

## Owner

Mantle project/source transport owner.

## Next action

Sync and archive this Cairn change after final lifecycle gates and repository validation pass.
