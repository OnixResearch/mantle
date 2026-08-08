# Mantle Durable Publication Promotion Specification Delta

## ADDED Requirements

### Requirement: Reviewed Mantle merge candidate

r[mantle.durable_publication_promotion.candidate]

One exact promotion candidate MUST preserve current canonical `main` and accepted Mantle commit `d1f3d6d96e2b0d9cd8497cd89b8e5a93d4a7dfaf` as ancestors. The candidate MUST use the fetched canonical target as its first-parent history and the accepted commit as its reviewed second-parent history. First-parent changes MUST remain limited to the named promotion lifecycle paths, the three adoption receipt files under `evidence/radicle/`, and `lib/durable-file-publication-adoption-receipt.ncl`. Receipt and validator changes MUST update only the canonical BLAKE3 bindings and receipt digest while preserving all accepted producer, mapping, authority, validation, and non-claim fields.

#### Scenario: Candidate preserves both histories

- GIVEN fresh Mantle remote refs
- WHEN merge-candidate ancestry is evaluated
- THEN the candidate MUST contain the canonical target and accepted Mantle commit.

#### Scenario: Target, accepted ancestry, or path scope drifts

- GIVEN a candidate that omits either history or changes a non-lifecycle first-parent path
- WHEN promotion is evaluated
- THEN mutation MUST remain blocked.

### Requirement: Upstream promotion order

r[mantle.durable_publication_promotion.ordering]

Mantle promotion MUST require Onix Core canonical `main` to contain reconciliation archive commit `bc4629c9e766d3db82e4dab9fe8c166c360b8435` and accepted admission commit `b8387cd7d59fa3b0d4ea67646352dd27c4f7d7ed`.

#### Scenario: Producer reconciliation is canonical

- GIVEN a fresh Onix Core remote observation
- WHEN canonical `main` contains both required commits
- THEN Mantle promotion MAY continue.

#### Scenario: Producer reconciliation or admission is feature-only

- GIVEN either required Onix Core commit is absent from canonical `main`
- WHEN Mantle promotion checks ordering
- THEN Mantle promotion MUST stop.

### Requirement: Non-destructive Mantle push

r[mantle.durable_publication_promotion.safe_push]

The shell MUST use a normal fast-forward push after an immediate fetch and explicit authorization. It MUST NOT force-push, rewrite history, create a pull request, or silently change the reviewed merge result.

#### Scenario: Authorized fast-forward succeeds

- GIVEN passing checks and unchanged ancestry
- WHEN the authorized push runs
- THEN Mantle `main` MUST advance without history rewriting.

#### Scenario: Remote changes before push

- GIVEN the remote target moves after validation
- WHEN the push is attempted
- THEN the push MUST fail without a force retry.

### Requirement: Mantle promotion evidence

r[mantle.durable_publication_promotion.validation]

Evidence MUST bind candidate and parent identities, producer-order observation, first-parent path scope, focused adoption checks, package probe, test results, traceability, push result, and final remote commit. Focused adoption checks MUST pass. Exact pre-existing broad failures MAY remain only when first-parent path evidence proves that the candidate did not change the affected implementation surface and the evidence records each failure without a success claim.

#### Scenario: Complete promotion evidence passes

- GIVEN matching observations from one candidate snapshot
- WHEN evidence is reviewed
- THEN the change MUST be eligible for synchronization and archive.

#### Scenario: Broad blockers are hidden or focused checks are missing

- GIVEN omitted focused results, a new broad failure, an affected implementation path, or inaccurate broad-check claims
- WHEN evidence is reviewed
- THEN the change MUST remain incomplete.
