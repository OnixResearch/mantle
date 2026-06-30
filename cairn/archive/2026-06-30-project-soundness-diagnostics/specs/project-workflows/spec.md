## ADDED Requirements

### Requirement: Mantle checks project workflow soundness [r[project_workflows.project_soundness_checks]]

Mantle MUST check static soundness across the project manifest, lockfile, generated inputs, patches, mirrors, hash algorithms, freshness metadata, trust policy, fetch policy, retention roots, and orphaned lock entries. Default soundness checks MUST be no-network unless an explicit mode documents freshness or trust execution.

#### Scenario: Static soundness checks clean project state [r[project_workflows.project_soundness_checks.scenario.clean]]

- GIVEN a project manifest, lockfile, generated inputs, patch definitions, mirrors, hash algorithms, fetch policy, trust policy, and retention roots agree
- WHEN `mantle check` runs in default mode
- THEN Mantle MUST report the static project state as sound
- AND it MUST NOT contact the network, run freshness commands, or fetch missing source material.

#### Scenario: Manifest lock mismatch is classified [r[project_workflows.project_soundness_checks.scenario.kind-mismatch]]

- GIVEN a manifest input and lockfile entry disagree on input kind, source identity, hash algorithm, patch set, fetch policy, or trust policy shape
- WHEN `mantle check` compares project state
- THEN Mantle MUST report a deterministic mismatch diagnostic naming the input and mismatched class
- AND it MUST NOT silently merge or refresh the entry.

#### Scenario: Generated inputs must match lockfile [r[project_workflows.project_soundness_checks.scenario.generated-stale]]

- GIVEN `.mantle/inputs.ncl` or equivalent generated input state is stale, missing, or inconsistent with the lockfile
- WHEN `mantle check` runs
- THEN Mantle MUST report a generated-input-stale diagnostic
- AND it MUST NOT rewrite generated inputs unless an explicit repair or refresh command is selected.

#### Scenario: Optional probe mode labels network behavior [r[project_workflows.project_soundness_checks.scenario.probe-mode]]

- GIVEN an operator explicitly requests soundness checks that execute freshness probes or trust verification
- WHEN those checks run
- THEN Mantle MUST label the mode and possible network or process behavior in diagnostics or JSON metadata
- AND failed probes or trust checks MUST be reported as soundness issues without rewriting lock entries.
