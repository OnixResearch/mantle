## ADDED Requirements

### Requirement: Project inputs may declare retention roots [r[project_workflows.input_retention_roots]]

Mantle MUST support a project-level default `retention` policy and per-input `retention` overrides for source/input material. Retention policy MUST distinguish `{ mode = "untracked" }`, `{ mode = "current" }`, and `{ mode = "recent-generations", generations = <positive bounded integer> }`; generation limits MUST be named, bounded, and validated before roots are treated as durable. Diagnostics MUST distinguish pinned, unpinned, stale-root, missing-root, and garbage-collection-eligible records.

#### Scenario: Current input is pinned [r[project_workflows.input_retention_roots.scenario.current]]

- GIVEN a project input has retention set to track the current locked source
- WHEN Mantle commits a refresh, import, or generated-input update for that input
- THEN Mantle MUST plan or create a retention root bound to the input name, lock digest, source identity, and content digest
- AND project diagnostics MUST report the input as pinned only after the root exists.

#### Scenario: Generation retention keeps bounded history [r[project_workflows.input_retention_roots.scenario.generations]]

- GIVEN a project default or input override retains recent generations
- WHEN Mantle updates the lockfile across multiple generations
- THEN Mantle MUST retain no more than the configured generation limit for that input
- AND generation selection MUST be based on Mantle-owned lock generation facts rather than filesystem timestamp ordering.

#### Scenario: Untracked input stays GC-eligible [r[project_workflows.input_retention_roots.scenario.untracked]]

- GIVEN a project input has retention disabled or untracked
- WHEN Mantle imports or refreshes that input
- THEN Mantle MUST report the source material as garbage-collection eligible unless another explicit root protects it
- AND it MUST NOT imply durable availability for branch switches or rebases.

### Requirement: Retention updates are atomic with project state [r[project_workflows.input_retention_atomicity]]

Mantle MUST update project retention roots atomically with the lock/source-state transitions they protect by committing same-directory temporary files into `.mantle/retention.json` and root marker records under `.mantle/retention-roots/`. Interrupted or partial retention updates, including `.mantle/retention.json.tmp`, MUST NOT be reported as durable roots for project readiness.

#### Scenario: Interrupted root update is not durable [r[project_workflows.input_retention_atomicity.scenario.interrupted]]

- GIVEN a refresh or import is interrupted after staging source bytes or root metadata
- WHEN Mantle checks project soundness or offline readiness
- THEN Mantle MUST treat uncommitted retention records as absent or quarantined
- AND it MUST NOT report the corresponding input as pinned.

#### Scenario: Stale root is diagnosed [r[project_workflows.input_retention_atomicity.scenario.stale-root]]

- GIVEN a retention root exists for an older lock digest or mismatched source digest outside the configured generation window
- WHEN Mantle checks project soundness
- THEN Mantle MUST report a stale-root diagnostic
- AND it MUST NOT count that root as satisfying current input retention.
