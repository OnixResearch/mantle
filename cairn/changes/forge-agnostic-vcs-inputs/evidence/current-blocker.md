# Current Blocker — Forge-agnostic VCS project inputs

Date: 2026-06-30

## Question

Can `forge-agnostic-vcs-inputs` be honestly drained from the current tree?

## Inspected evidence

- `cairn/changes/forge-agnostic-vcs-inputs/tasks.md` still has 9 unchecked tasks covering Darcs/Pijul/Fossil schemas, fail-closed behavior, pure selector/lock/mirror/generated-input logic, shell fetcher adapters, integration with freshness/fetch policy/source bundles, and positive/negative/fixture tests.
- The spec requires VCS-native identities plus locked source-tree content digests for Darcs, Pijul, and Fossil, and deterministic blockers for missing tools or unsupported subfeatures.
- Current project input support is Git/file/archive oriented; code search did not find Darcs/Pijul/Fossil project input types or fetcher adapters.
- The change depends on still-active freshness probes, fetch policy, and source-bundle/source-state work to provide complete refresh, mirror, source bundle, and generated-input behavior.
- External tool availability and exact identity extraction strategy for Darcs/Pijul/Fossil still need explicit implementation decisions before fixture tests can claim support.

## Decision

Blocked. This is a valid scope package, but support cannot be honestly claimed until VCS identity extraction, adapter strategy, source-bundle integration, and fail-closed fixture tests exist.

## Owner

Mantle project/source transport owner after freshness/fetch-policy/source-bundle foundations are ready.

## Next action

1. choose the first VCS implementation slice and fixture strategy, preferably one local deterministic adapter at a time;
2. add pure selector and lock metadata validation for that VCS;
3. implement shell fetch/materialization with deterministic missing-tool blockers;
4. integrate with freshness/fetch-policy/source-bundle paths before expanding to the next VCS.
