## ADDED Requirements

### Requirement: Project inputs support forge-agnostic VCS kinds [r[project_workflows.forge_agnostic_vcs_inputs]]

Mantle MUST support project input kinds for Darcs, Pijul, and Fossil without privileged forge-specific URL schemes. Each VCS kind MUST use explicit repository or remote URLs, VCS-native selectors, mirrors when verifiable, locked VCS identity metadata, and a locked source-tree content digest.

#### Scenario: Darcs input locks native identity [r[project_workflows.forge_agnostic_vcs_inputs.scenario.darcs]]

- GIVEN a project input declares a Darcs repository with a tag or context selector
- WHEN Mantle refreshes or locks the input
- THEN Mantle MUST record Darcs-native identity metadata such as context or weak-hash facts when available
- AND it MUST record a source-tree content digest for the materialized checkout.

#### Scenario: Pijul input locks channel state [r[project_workflows.forge_agnostic_vcs_inputs.scenario.pijul]]

- GIVEN a project input declares a Pijul remote with channel, state, or change selection
- WHEN Mantle refreshes or locks the input
- THEN Mantle MUST record the selected Pijul identity facts
- AND it MUST verify mirrors or refreshed material against the same locked source-tree content digest.

#### Scenario: Fossil input locks check-in identity [r[project_workflows.forge_agnostic_vcs_inputs.scenario.fossil]]

- GIVEN a project input declares a Fossil repository with branch, tag, or check-in selection
- WHEN Mantle refreshes or locks the input
- THEN Mantle MUST record the selected Fossil identity facts
- AND it MUST produce generated input data that refers to the locked source material without requiring forge-specific semantics.

#### Scenario: Forge shortcuts are not source semantics [r[project_workflows.forge_agnostic_vcs_inputs.scenario.no-forge-shortcut]]

- GIVEN a repository is hosted on GitHub, GitLab, Codeberg, a Darcs hub, a Pijul nest, or a Fossil host
- WHEN Mantle validates the project input
- THEN Mantle MUST treat the host as a URL endpoint only
- AND it MUST NOT require or privilege host-specific shorthand schemes as part of source identity.

### Requirement: VCS inputs fail closed when identity cannot be proven [r[project_workflows.vcs_input_fail_closed]]

Mantle MUST fail closed when a non-Git VCS input cannot prove the selected source identity, content digest, or mirror equivalence. Missing tools or unsupported VCS subfeatures MUST be reported as deterministic blockers rather than successful source support.

#### Scenario: Missing VCS tool is an unsupported blocker [r[project_workflows.vcs_input_fail_closed.scenario.missing-tool]]

- GIVEN a project input requires a VCS adapter whose implementation or external tool is unavailable
- WHEN Mantle plans, refreshes, or builds the input
- THEN Mantle MUST emit an unsupported-VCS-tool diagnostic
- AND it MUST NOT update the lockfile or report the input as fetched.

#### Scenario: Mirror mismatch is rejected [r[project_workflows.vcs_input_fail_closed.scenario.mirror-mismatch]]

- GIVEN a VCS input mirror resolves to material that does not match the locked VCS identity or source-tree digest
- WHEN Mantle verifies the mirror result
- THEN Mantle MUST reject that mirror result
- AND it MUST continue to another mirror only if the next result can prove the same identity.
