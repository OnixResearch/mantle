# Validation evidence

## Baseline

The clean task-owned artifact-auth commit `f986ce20` and pre-adoption parent `52a0c657` both fail `checks.x86_64-linux.nextest` after 263 passing tests at `mantle::examples_inventory::flake_source_keeps_project_and_transcript_support_trees`. The panic is `tests/examples_inventory.rs:190` while reading a missing repository file. Host execution passes because the working checkout contains `flake.nix`; inspection of the shared filter confirms it includes project/transcript support but not `flake.nix` itself.

The working tree also contains independent `promote-full-source-seed-provider` files and bootstrap changes. This change must not edit, stage, commit, validate as its own, or suppress any of that work. Completion requires exact file inclusion, continued Git-metadata exclusion, focused checks, and clean-commit Nix evidence.

## Focused result

The shared filter now admits only the exact `flake.nix` path in addition to its prior inventory. The focused test requires positive flake self-description and negative exact/prefix Git-metadata exclusions. Focused host execution, package rustfmt, strict first-party Clippy, and the repository Tiger Style rail pass. No Cargo/Nix dependency or lock changed.

Clean task commit `2565e836` passes `checks.x86_64-linux.nextest`, advancing through the former 263-test failure and completing the full partition. Full `nix flake check` against the same immutable local Git revision also passes on `x86_64-linux`. This validates only filtered-source completeness and repository checks; it does not claim bootstrap/source-seed correctness or include the unrelated dirty worktree.
