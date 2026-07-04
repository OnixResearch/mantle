# Design: Project file generation

## Architecture

File generation is split into a pure planner and a filesystem shell.

- Pure core: receives project manifest file declarations plus explicit current-file facts. It validates targets, materialization methods, contract bindings, expected content digests, and conflict policy, then returns a deterministic `FilegenPlan`.
- Imperative shell: reads existing files, invokes Nickel export/evaluation when needed, writes copies or symlinks during apply, and renders human/JSON diagnostics.

The pure core never reads or writes the filesystem. It only compares declared paths, content refs, hashes, and current-file facts supplied by the shell.

## Declaration model

Each generated file declaration should include a stable name, target path, content source, materialization method, and optional contract/schema metadata. Initial content sources can be inline text or a Nickel export action. Initial materialization methods can be copy and symlink. Target paths must normalize under the project root and must not collide unless the plan identifies an accepted replacement.

## Plan/apply flow

`mantle filegen plan` gathers current-file facts, computes desired content, and reports create/update/unchanged/conflict operations without mutation. `mantle filegen apply` recomputes or verifies the plan, refuses drift, and writes only the operations present in the accepted plan.

Generated outputs should carry BLAKE3 content digests so review, apply, and later evidence can prove whether the file content changed.

## Validation strategy

Pure tests should cover path normalization, conflict classification, content digest planning, materialization method validation, stale generated files, and drift detection. Shell tests should cover no-mutate plan behavior, apply writes, symlink/copy behavior, target escape rejection, existing-file conflicts, and JSON output parseability.
