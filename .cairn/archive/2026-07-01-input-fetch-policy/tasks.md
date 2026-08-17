# Tasks

## Contract

- [x] [serial] Define project input fetch policy schema, defaults, named policy classes, and compatibility rules for generation-time, build-time, and imported/offline source requirements. r[project_workflows.input_fetch_policy]
- [x] [serial] Define fetch policy diagnostics and offline preflight behavior for network-required, source-state-required, build-fetch-required, unsupported, and conflicting policies. r[project_workflows.input_fetch_policy_preflight]

## Implementation

- [x] [serial] Implement pure fetch policy validation, requirement classification, generated-input planning, and unsupported-combination diagnostics. r[project_workflows.input_fetch_policy]
- [x] [serial] Thread policy through refresh, generated `.mantle/inputs.ncl`, build planning, source bundle planning, and offline preflight shells. r[project_workflows.input_fetch_policy] r[project_workflows.input_fetch_policy_preflight]

## Verification

- [x] [serial] Add positive pure tests for policy defaults, per-input overrides, patched input compatibility, build-time fetch lowering classification, and imported source-state requirements. r[project_workflows.input_fetch_policy]
- [x] [serial] Add negative pure tests for unknown policies, patched inputs in incompatible modes, evaluation-required inputs without source material, offline network requirements, and policy conflicts. r[project_workflows.input_fetch_policy] r[project_workflows.input_fetch_policy_preflight]
- [x] [serial] Add CLI tests proving check/list-stale do not fetch by default, refresh mutates only in explicit modes, and offline preflight blocks before sandbox execution. r[project_workflows.input_fetch_policy_preflight]
- [x] [serial] Run `nix run path:/home/brittonr/git/cairn#cairn -- validate --root .` and proposal/design/tasks gates before marking implementation tasks complete. r[project_workflows.input_fetch_policy]
