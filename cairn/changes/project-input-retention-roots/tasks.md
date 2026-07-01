# Tasks

## Contract

- [x] [serial] Define project input retention policy schema, default/per-input overrides, generation limits, lock-generation binding, and GC-eligible reporting. r[project_workflows.input_retention_roots]
- [x] [serial] Define atomic retention update semantics for lock/source-state/root changes and interrupted update diagnostics. r[project_workflows.input_retention_atomicity]

## Implementation

- [x] [serial] Implement pure retention policy validation, generation selection, root action planning, stale-root detection, and GC-eligible classification. r[project_workflows.input_retention_roots]
- [x] [serial] Implement shell persistence for retention roots and project retention state with atomic update behavior. r[project_workflows.input_retention_atomicity]
- [x] [serial] Surface pinned, unpinned, stale-root, missing-root, and GC-eligible states in project diagnostics. r[project_workflows.input_retention_roots]

## Verification

- [x] [serial] Add positive pure tests for default policy, per-input override, generation cutoff, root action ordering, stale-root detection, and lock digest binding. r[project_workflows.input_retention_roots]
- [x] [serial] Add negative pure tests for invalid generation limits, roots for unknown inputs, mismatched lock digests, interrupted updates, and unpinned records reported as GC-eligible. r[project_workflows.input_retention_roots] r[project_workflows.input_retention_atomicity]
- [x] [serial] Add shell tests with temp store/source state proving roots are written atomically and undeclared inputs are not retained. r[project_workflows.input_retention_atomicity]
- [x] [serial] Run `nix run path:/home/brittonr/git/cairn#cairn -- validate --root .` and proposal/design/tasks gates before marking implementation tasks complete. r[project_workflows.input_retention_roots]
