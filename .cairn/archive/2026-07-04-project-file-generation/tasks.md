# Tasks

## Contract

- [x] [serial] Define generated-file declarations, target path policy, content source kinds, materialization methods, plan/apply semantics, and no-mutate behavior. r[project_workflows.project_filegen]
- [x] [serial] Define typed content validation and schema/contract binding for generated files without claiming frontend deployability. r[project_workflows.project_filegen_typed_content]

## Implementation

- [x] [serial] Implement pure filegen plan normalization, target validation, content digest comparison, conflict detection, and drift checks. r[project_workflows.project_filegen]
- [x] [serial] Implement `mantle filegen plan` and `mantle filegen apply` shells with copy/symlink materialization and stable human/JSON diagnostics. r[project_workflows.project_filegen]
- [x] [serial] Integrate Nickel export receipts or content digests into filegen output evidence. r[project_workflows.project_filegen_typed_content]

## Verification

- [x] [serial] Add positive tests for no-mutate plan, create/update/unchanged planning, apply writes, copy materialization, symlink materialization, and typed valid content. r[project_workflows.project_filegen] r[project_workflows.project_filegen_typed_content]
- [x] [serial] Add negative tests for target path escapes, existing-file conflicts, plan drift, unsupported materialization methods, stale generated files, and contract-invalid content. r[project_workflows.project_filegen] r[project_workflows.project_filegen_typed_content]
- [x] [serial] Run `nix run path:/home/brittonr/git/cairn#cairn -- validate --root .` and proposal/design/tasks gates before implementation tasks are marked complete. r[project_workflows.project_filegen]

Evidence: `nix develop -c cargo test -p crunch-project-core --lib`, `nix develop -c cargo test -p mantle --bin mantle filegen_cmd`, temp-directory `mantle --json filegen plan/apply` smoke, and Cairn proposal/design/tasks gates passed for this change.
