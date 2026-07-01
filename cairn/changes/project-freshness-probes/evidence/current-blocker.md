# Current Blocker — Project freshness probes

Date: 2026-06-30

## Question

Can `project-freshness-probes` be honestly drained from the current tree?

## Inspected evidence

- `cairn/changes/project-freshness-probes/tasks.md` still has 9 unchecked tasks covering probe schema, observation records, pure classification/template planning, Git/HTTP/local/command shell adapters, refresh/list-stale lock integration, and positive/negative/CLI fixture tests.
- The proposal and design require a bounded command probe contract, HTTP text/JSON probes, Git reference probes, local file/directory probes, no-network behavior, and template substitution that can feed refresh plans.
- Current project code supports refresh/list-stale through `RefreshResolver`, but code search did not find implemented freshness probe types, observation records, command probe bounds, HTTP/local probe shells, or lockfile observation persistence.
- Several sibling project-input changes depend on freshness semantics (`project-lock-importers`, `forge-agnostic-vcs-inputs`, and `input-fetch-policy`), so a partial hidden implementation would create cross-change overclaims.

## Decision

Blocked as a complete drain. The change needs a new project-input observation model plus multiple shell adapters and fixture tests; none of that support is present yet.

## Owner

Mantle project workflow owner for the freshness-observation model and adapter shell.

## Next action

1. implement the pure observation/classification/template core with positive and negative tests;
2. add shell adapters incrementally, starting with local file/directory and Git fixtures before HTTP/command probes;
3. thread observations into list-stale and refresh reports without mutating on list-stale;
4. keep network/no-network behavior explicit before enabling dependent importer/fetch-policy claims.
