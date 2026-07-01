# Tasks

## Contract

- [ ] [serial] Define Darcs, Pijul, and Fossil project input schemas, selectors, lock metadata, mirror semantics, generated input rendering, and forge-neutral URL rules. r[project_workflows.forge_agnostic_vcs_inputs]
- [ ] [serial] Define fail-closed behavior for missing VCS tooling, unresolved identity facts, mirror mismatch, unsupported selectors, and unavailable source tree digests. r[project_workflows.vcs_input_fail_closed]

## Implementation

- [ ] [serial] Implement pure VCS input types, selector normalization, lock metadata validation, mirror compatibility planning, and generated-input rendering data. r[project_workflows.forge_agnostic_vcs_inputs]
- [ ] [serial] Implement shell fetcher adapters for supported Darcs, Pijul, and Fossil fixtures through Mantle source/fetch services. r[project_workflows.forge_agnostic_vcs_inputs] r[project_workflows.vcs_input_fail_closed]
- [ ] [serial] Integrate VCS kinds with freshness probes, fetch policy, source bundles when available, and project refresh/list-stale. r[project_workflows.forge_agnostic_vcs_inputs]

## Verification

- [ ] [serial] Add positive pure tests for selectors, lock metadata, mirrors, generated input data, and content digest binding. r[project_workflows.forge_agnostic_vcs_inputs]
- [ ] [serial] Add negative pure tests for ambiguous selectors, missing identity facts, mirror mismatch, unsupported subfeatures, stale lock entries, and forge-specific shorthand rejection. r[project_workflows.forge_agnostic_vcs_inputs] r[project_workflows.vcs_input_fail_closed]
- [ ] [serial] Add fixture tests using local repositories or deterministic tool-availability blockers for Darcs, Pijul, and Fossil. r[project_workflows.vcs_input_fail_closed]
- [ ] [serial] Run `nix run path:/home/brittonr/git/cairn#cairn -- validate --root .` and proposal/design/tasks gates before marking implementation tasks complete. r[project_workflows.forge_agnostic_vcs_inputs]
