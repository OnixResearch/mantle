## ADDED Requirements

### Requirement: Project inputs declare explicit fetch policy [r[project_workflows.input_fetch_policy]]

Mantle MUST let project inputs declare an explicit fetch policy describing when source material may be acquired or required. The policy model MUST distinguish source material needed before generated inputs or dependent evaluation, source material lowered into ordinary build-time fetch actions, and source material that must already be present in imported/offline source state.

#### Scenario: Policy classifies fetch requirements [r[project_workflows.input_fetch_policy.scenario.classify]]

- GIVEN a project manifest declares inputs with different fetch policies
- WHEN Mantle plans refresh, generated inputs, build actions, or source bundle requirements
- THEN Mantle MUST classify each input as generation-material, build-fetch-action, imported-source-required, already-present, unsupported, or conflicting
- AND the classification MUST be deterministic for the same manifest, lockfile, and source-state facts.

#### Scenario: Incompatible policy fails closed [r[project_workflows.input_fetch_policy.scenario.incompatible]]

- GIVEN an input has patches, trust policy, template dependencies, or evaluation-time consumers that cannot be honored by its selected fetch policy
- WHEN Mantle validates the manifest or plans the input
- THEN Mantle MUST reject the incompatible policy with deterministic diagnostics
- AND it MUST NOT silently switch to a broader network or fetch mode.

#### Scenario: Policy remains build-tool data [r[project_workflows.input_fetch_policy.scenario.boundary]]

- GIVEN a frontend or project supplies input fetch policy
- WHEN Mantle validates project workflow data
- THEN Mantle MUST treat the policy as frontend-neutral build/source data
- AND it MUST NOT interpret Onix, NixOS, flake output, or module-layer semantics to decide the policy.

### Requirement: Fetch policy controls offline preflight [r[project_workflows.input_fetch_policy_preflight]]

Mantle MUST apply input fetch policy during offline preflight before sandbox execution or remote dispatch. Offline preflight MUST fail closed when a selected build would require network access or unavailable source state under the declared policy.

#### Scenario: Imported source satisfies offline policy [r[project_workflows.input_fetch_policy_preflight.scenario.imported-ready]]

- GIVEN an input's policy requires imported source state
- AND local source state contains a matching source record with the locked digest and identity
- WHEN Mantle runs offline preflight for a selected root
- THEN Mantle MAY classify that input as ready for an offline build attempt
- AND the report MUST bind the source-state or source-bundle digest used for the decision.

#### Scenario: Network-required policy blocks offline build [r[project_workflows.input_fetch_policy_preflight.scenario.network-blocked]]

- GIVEN an input policy would require generation-time or build-time network fetches that are not satisfied by imported source state
- WHEN Mantle runs offline preflight
- THEN Mantle MUST fail before sandbox execution or remote dispatch
- AND diagnostics MUST identify the input, selected policy, and missing source-state class.
