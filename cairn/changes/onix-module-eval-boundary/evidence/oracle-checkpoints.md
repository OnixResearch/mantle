# Oracle checkpoints

## Excluding `src/cargo_free_self_build.rs` from system-eval removal commits

Question: Should `src/cargo_free_self_build.rs` be included in the Mantle system-eval removal / build-tool-boundary commits?

Inspected evidence:

- The user-provided status for the commit turn listed `src/cargo_free_self_build.rs` under staged changes.
- The compacted turn context said `src/cargo_free_self_build.rs` was already dirty and unrelated before the system-eval removal work, and should be left untouched.
- `git diff --cached -- src/cargo_free_self_build.rs` showed rustc wrapper/probe and formatting changes, not system-eval removal, docs, stdlib, Cairn boundary, or CLI-surface changes.
- After committing `8269ed68`, `git status --short` still showed only ` M src/cargo_free_self_build.rs`, confirming it remained an uncommitted worktree change outside the boundary-removal commit.

Decision: Exclude `src/cargo_free_self_build.rs` from the system-eval removal and scope-guard commits. It is separate cargo-free self-build work, not part of the Mantle/Onix module-boundary change.

Owner: Britton/user owns whether to keep, revert, or commit that cargo-free self-build change in a separate follow-up.

Next action: Leave `src/cargo_free_self_build.rs` dirty and uncommitted until the owner requests that separate follow-up. Before any future commit, re-check `git status --short` and explicitly stage only files relevant to the requested change.
