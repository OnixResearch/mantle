## ADDED Requirements

### Requirement: Project shell profiles are named build-tool handoffs [r[project_workflows.named_shell_profiles]]

Mantle MUST support named project shell profiles that resolve to explicit build inputs, environment entries, path entries, and activation sidecar data. Shell profiles MUST remain build-tool handoffs and MUST NOT cause Mantle core to interpret Nix flakes, Onix modules, service lifecycle semantics, or frontend-specific package sets.

#### Scenario: Default shell profile resolves deterministically [r[project_workflows.named_shell_profiles.scenario.default]]

- GIVEN a project declares `build`, `dev`, and optionally `default` shell profiles
- WHEN an operator runs `mantle shell` without an explicit profile name
- THEN Mantle MUST resolve the default profile deterministically
- AND diagnostics MUST identify which named profile was selected.

#### Scenario: Explicit profile lowers to activation plan [r[project_workflows.named_shell_profiles.scenario.explicit]]

- GIVEN a project declares a named shell profile with explicit build inputs, environment entries, and path entries
- WHEN an operator runs `mantle shell <name>` or an equivalent non-interactive shell command
- THEN Mantle MUST lower that profile to ordinary Mantle build inputs and activation sidecar data
- AND the pure shell-planning core MUST receive owned normalized data rather than reading the filesystem, environment, or frontend state.

#### Scenario: Invalid profile fails before activation [r[project_workflows.named_shell_profiles.scenario.invalid]]

- GIVEN a profile name is invalid, the default is ambiguous, env entries collide after normalization, path entries are unsupported, or adapter paths cannot be represented safely
- WHEN Mantle validates or activates the shell profile
- THEN Mantle MUST fail with deterministic diagnostics before executing a shell or hook
- AND it MUST NOT silently fall back to another profile.

#### Scenario: Services remain outside shell profile semantics [r[project_workflows.named_shell_profiles.scenario.no-services]]

- GIVEN a project or frontend wants long-running development services such as databases, queues, or daemons
- WHEN Mantle processes named shell profiles
- THEN Mantle MUST reject service lifecycle declarations or treat service descriptors only as opaque generated data under a separate explicit contract
- AND it MUST NOT start, stop, supervise, restart, or health-check services as part of shell profile activation.

#### Scenario: Shell profile claim is bounded [r[project_workflows.named_shell_profiles.scenario.non-claim]]

- GIVEN a named shell profile activates successfully
- WHEN Mantle renders human output, JSON output, docs, or evidence
- THEN the claim MAY state that the selected shell activation plan was produced and applied under recorded inputs
- AND it MUST NOT claim build success, test success, service readiness, deployability, or release reproducibility without separate evidence.

### Requirement: Dev shell activation is decoupled from build identity [r[project_workflows.dev_shell_decoupling]]

Mantle MUST keep dev shell activation separate from package build action identity, file generation, lock refresh, release evidence, and reproducibility claims. A dev shell MAY reuse Mantle-built packages, but entering or planning the shell MUST NOT change build hashes, generated files, lockfiles, project manifests, or proof state unless an operator invokes a separate explicit mutating command.

#### Scenario: Dev shell does not affect action identity [r[project_workflows.dev_shell_decoupling.scenario.action-identity]]

- GIVEN a project has package build declarations and a `dev` shell profile with extra convenience tools or environment entries
- WHEN Mantle computes package action specs before and after planning or activating the dev shell
- THEN the package action identity MUST remain determined by the package declarations and declared build inputs
- AND `shells.dev` MUST NOT become an implicit input to package build hashes.

#### Scenario: Shell activation is non-mutating by default [r[project_workflows.dev_shell_decoupling.scenario.non-mutating]]

- GIVEN a project has generated files, lock entries, refreshable inputs, and named shell profiles
- WHEN an operator runs `mantle shell` or `mantle shell dev`
- THEN Mantle MUST NOT regenerate files, rewrite lockfiles, refresh inputs, edit project manifests, or mutate release/proof evidence as part of shell activation
- AND any required mutation MUST be routed through a separate explicit command such as file generation, refresh, or build.

#### Scenario: Shell activation receipt is separate evidence [r[project_workflows.dev_shell_decoupling.scenario.separate-receipt]]

- GIVEN Mantle records evidence for shell activation
- WHEN human output, JSON output, release evidence, or task evidence cites that shell evidence
- THEN the evidence MUST be a separate shell-activation receipt or equivalent bounded record naming the selected profile and activation plan
- AND build reports, release evidence, and reproducibility claims MUST NOT treat that receipt as build or release proof unless a separate requirement explicitly admits the narrower claim.

#### Scenario: Dev shell overclaiming is rejected [r[project_workflows.dev_shell_decoupling.scenario.non-claim]]

- GIVEN a dev shell activates successfully
- WHEN Mantle renders diagnostics, docs, release notes, task evidence, or final status
- THEN the claim MAY state only that the selected dev shell activation plan was applied or made available
- AND it MUST NOT claim package build success, test success, file generation freshness, lock freshness, service readiness, release reproducibility, or build action correctness without separate current evidence.
