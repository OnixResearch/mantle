# Design: preserve flake source inventory

## Completion contract

The change is complete when the shared filtered source includes `flake.nix`, continues excluding `.git` metadata, the focused host inventory test passes, and the previously failing clean-commit Nix `nextest` check advances past `flake_source_keeps_project_and_transcript_support_trees`. Editing tests to avoid reading the flake, disabling the test, including all repository files, or relying on the unrelated dirty bootstrap tree is false completion.

## Pure filter boundary

The existing `cleanSourceWith` predicate remains the source-selection core. Add one exact path equality for `flake.nix`; do not widen to a directory or ambient path. The test asserts the positive inclusion rule and the negative Git-metadata exclusion rule.

## Validation isolation

The working tree contains separate source-seed work. Commit only this lifecycle package, `flake.nix`, and the inventory assertion, then run Nix against that clean commit identity. Preserve all unrelated staged, modified, and untracked files.
