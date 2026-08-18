# Tasks

## Contract

- [x] [serial] Define the Cargo import plan schema, supported initial Cargo surfaces, file-operation model, conflict classes, and no-mutate/apply contract. r[project_workflows.cargo_import_scaffold]
  - Evidence: `src/cargo_import.rs` defines `mantle-cargo-import-plan-v1`, file operations with BLAKE3 digests, blockers, non-claims, and separate plan/apply shell behavior.
- [x] [serial] Document that import emits build-shaped Mantle project data and does not introduce module-layer semantics or Cargo-free claims. r[project_workflows.cargo_import_scaffold]
  - Evidence: `README.md` and `docs/operator-workflows.md` document `mantle import cargo --plan|--apply`, generated build-shaped files, and no Cargo-free/module-layer claims.

## Implementation

- [x] [serial] Add a pure `CargoImportPlan` builder over normalized workspace/package/lock/source facts. r[project_workflows.cargo_import_scaffold]
  - Evidence: pueue task 249 passed five `cargo_import` unit tests over the pure planner and shell boundary.
- [x] [serial] Add the CLI shell for planning and applying the scaffold while keeping `--plan` side-effect free. r[project_workflows.cargo_import_scaffold]
  - Evidence: `Command::Import` wires `mantle import cargo`; pueue task 243 passed the CLI no-mutate plan/apply/conflict tests.
- [x] [serial] Generate Mantle-canonical project files and `.mantle/inputs.ncl` with deterministic Nickel field quoting and content digests. r[project_workflows.cargo_import_scaffold]
  - Evidence: `src/cargo_import.rs` renders `mantle-project.ncl` plus `.mantle/inputs.ncl`; pueue task 243 asserted generated file paths, BLAKE3 digests, and project file shape.
- [x] [serial] Wire generated package outputs to the offline Cargo build package contract. r[project_workflows.cargo_import_scaffold]
  - Evidence: generated `mantle-project.ncl` uses `mantle.offlineCargoPackage`; `src/project_build.rs` now discovers canonical `mantle-project.ncl` while preserving legacy `crunch.ncl`.

## Verification

- [x] [serial] Add positive planner tests for a local workspace with one binary and declared local/vendored dependency source facts. r[project_workflows.cargo_import_scaffold]
  - Evidence: pueue task 249 passed `cargo_import_plan_generates_build_shaped_files_for_local_binary_workspace`.
- [x] [serial] Add positive apply tests that write a temp scaffold and validate generated Nickel/project shape. r[project_workflows.cargo_import_scaffold]
  - Evidence: pueue task 243 passed `cargo_import_apply_writes_bounded_files_and_project_shape`.
- [x] [serial] Add negative tests for file conflicts, missing lockfile, unsupported dependency source, ambiguous default package, malformed names, and non-UTF-8 paths. r[project_workflows.cargo_import_scaffold]
  - Evidence: pueue task 249 covered ambiguous/malformed/non-UTF-8 planner negatives; pueue task 243 covered CLI file conflicts, missing lockfile, and unsupported registry source.
- [x] [serial] Add a focused end-to-end smoke that imports a fixture and builds it through the offline Cargo lane, or records the narrower blocker if the offline lane is not complete yet. r[project_workflows.cargo_import_scaffold]
  - Evidence/blocker: pueue task 243 imports/applies a fixture and validates the generated project handoff, but full build-through is intentionally narrower because the scaffold emits source/toolchain placeholders rather than materializing admitted inputs; the offline Cargo lane itself is covered by pueue task 177.
- [x] [serial] Update README/operator docs and run Cairn validation plus focused project import tests. r[project_workflows.cargo_import_scaffold]
  - Evidence: docs updated; pueue tasks 243, 249, and 223 passed focused import/offline tests. Cairn validation follows this task update.
