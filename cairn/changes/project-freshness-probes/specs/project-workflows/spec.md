## ADDED Requirements

### Requirement: Project inputs declare bounded freshness probes [r[project_workflows.freshness_probes]]

Mantle MUST support versioned, bounded freshness probes for project inputs and patches. Probe definitions MUST be manifest data, and probe execution MUST stay in the imperative shell while the functional core receives normalized observation records. Supported probe families MUST include built-in Git reference observations, HTTP text or JSON observations, local file or directory observations, and explicitly bounded command observations.

#### Scenario: Built-in probe produces normalized observation [r[project_workflows.freshness_probes.scenario.builtin]]

- GIVEN a project input declares a supported built-in freshness probe
- WHEN Mantle evaluates freshness for that input
- THEN the shell MUST produce a normalized observation containing the input name, probe kind, observed value, value digest, status, and bounded diagnostics
- AND the pure core MUST classify the observation without reading the network, filesystem, environment, or process state.

#### Scenario: Command probe is bounded [r[project_workflows.freshness_probes.scenario.command-bounded]]

- GIVEN a project input declares a command freshness probe
- WHEN Mantle runs the probe
- THEN Mantle MUST enforce explicit argv, cwd, environment, timeout, output-size, and success-status limits
- AND empty output, oversized output, timeout, missing executable, invalid UTF-8 where UTF-8 is required, or non-success status MUST become deterministic probe failures.

#### Scenario: Offline mode does not run network probes [r[project_workflows.freshness_probes.scenario.offline]]

- GIVEN a freshness probe requires network access
- WHEN Mantle runs list-stale, refresh planning, check, or offline preflight in a no-network mode
- THEN Mantle MUST report the probe as network-required or unavailable
- AND it MUST NOT contact the network or silently treat the old lock value as freshly observed.

### Requirement: Refresh uses freshness observations without overclaiming [r[project_workflows.freshness_probe_refresh]]

Mantle MUST use normalized freshness observations to drive `mantle list-stale` and `mantle refresh` decisions. Freshness MUST determine whether a locked input should be considered stale, but it MUST NOT by itself prove source integrity, trust, build success, or reproducibility.

#### Scenario: List stale does not mutate [r[project_workflows.freshness_probe_refresh.scenario.list-stale]]

- GIVEN project inputs have existing lockfile freshness values
- WHEN `mantle list-stale` compares current observations against the lockfile
- THEN Mantle MUST report stale, unchanged, failed, skipped, or network-required inputs deterministically
- AND it MUST NOT write the lockfile, generated inputs, store state, or retention roots.

#### Scenario: Refresh updates only selected stale inputs [r[project_workflows.freshness_probe_refresh.scenario.refresh-selected]]

- GIVEN selected project inputs have valid freshness observations and at least one differs from the lockfile
- WHEN `mantle refresh` runs for those inputs
- THEN Mantle MUST fetch and hash only selected stale inputs plus required patches or trust material
- AND unchanged or failed inputs MUST remain unchanged in the lockfile unless an explicit repair mode is selected.

#### Scenario: Freshness value may feed a bounded template [r[project_workflows.freshness_probe_refresh.scenario.template]]

- GIVEN an input URL or reference template uses the validated freshness value
- WHEN Mantle renders the fetch plan
- THEN Mantle MUST render the template through bounded pure logic
- AND undefined variables, invalid rendered syntax, or oversized rendered values MUST fail before fetch or lock update.
