# Mantle Durable Publication Promotion Specification Delta

## ADDED Requirements

### Requirement: Reviewed Mantle candidate

r[mantle.durable_publication_promotion.candidate]

One exact promotion candidate hash MUST descend from accepted Mantle commit `d1f3d6d96e2b0d9cd8497cd89b8e5a93d4a7dfaf`. The fetched canonical target MUST be an ancestor. Every later candidate commit MUST be reviewed and limited to the named Cairn planning and evidence roots before promotion.

#### Scenario: Candidate remains a fast-forward

- GIVEN fresh Mantle remote refs
- WHEN candidate ancestry is evaluated
- THEN the candidate MUST contain the canonical target and accepted Mantle commit.

#### Scenario: Target or candidate ancestry drifts

- GIVEN a divergent target or candidate that omits the accepted commit
- WHEN promotion is evaluated
- THEN mutation MUST remain blocked.

### Requirement: Upstream promotion order

r[mantle.durable_publication_promotion.ordering]

Mantle promotion MUST require Onix Core canonical `main` to contain accepted admission commit `b8387cd7d59fa3b0d4ea67646352dd27c4f7d7ed`.

#### Scenario: Producer milestone is canonical

- GIVEN a fresh Onix Core remote observation
- WHEN its canonical branch contains the accepted admission commit
- THEN Mantle promotion MAY continue.

#### Scenario: Producer milestone is feature-only

- GIVEN the accepted Onix Core commit is absent from canonical `main`
- WHEN Mantle promotion checks ordering
- THEN Mantle promotion MUST stop.

### Requirement: Non-destructive Mantle push

r[mantle.durable_publication_promotion.safe_push]

The shell MUST use a normal fast-forward push after an immediate fetch and explicit authorization. It MUST NOT force-push, rewrite history, create a pull request, or silently merge remote changes.

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

Evidence MUST bind candidate and remote identities, producer-order observation, focused adoption checks, package probe, test results, traceability, push result, and final remote commit.

#### Scenario: Complete promotion evidence passes

- GIVEN matching observations from one candidate snapshot
- WHEN evidence is reviewed
- THEN the change MUST be eligible for synchronization and archive.

#### Scenario: Broad blockers are hidden or focused checks are missing

- GIVEN omitted focused results or inaccurate broad-check claims
- WHEN evidence is reviewed
- THEN the change MUST remain incomplete.
