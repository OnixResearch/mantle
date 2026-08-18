# Tasks

## Contract

- [x] [serial] Define freshness probe schema, observation records, supported probe kinds, value limits, template substitution rules, and offline/no-network behavior. r[project_workflows.freshness_probes]
- [x] [serial] Define list-stale and refresh semantics for stale/unchanged/failed observations without treating freshness as integrity proof. r[project_workflows.freshness_probe_refresh]

## Implementation

- [x] [serial] Implement pure freshness probe core types, observation normalization, stale classification, template render planning, and bounded diagnostics. r[project_workflows.freshness_probes] r[project_workflows.freshness_probe_refresh]
- [x] [serial] Implement shell adapters for Git, HTTP text/JSON, local file/directory, and bounded command probes. r[project_workflows.freshness_probes]
- [x] [serial] Thread observations through `mantle list-stale`, `mantle refresh`, lock updates, and generated input updates. r[project_workflows.freshness_probe_refresh]

## Verification

- [x] [serial] Add positive pure tests for built-in probe observations, command observation normalization, stale classification, selected refresh, and template substitution. r[project_workflows.freshness_probes] r[project_workflows.freshness_probe_refresh]
- [x] [serial] Add negative pure tests for empty values, oversized values, invalid templates, failed observations, network-required offline observations, and input-name mismatches. r[project_workflows.freshness_probes]
- [x] [serial] Add shell/CLI fixture tests for local Git, local HTTP, local directory, command timeout, command missing, output limit, and no-network mode. r[project_workflows.freshness_probes] r[project_workflows.freshness_probe_refresh]
- [x] [serial] Run `nix run path:/home/brittonr/git/cairn#cairn -- validate --root .` and proposal/design/tasks gates before marking implementation tasks complete. r[project_workflows.freshness_probes]
