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

## Resolving final `src/cargo_free_self_build.rs` dirty-state ambiguity

Question: The earlier checkpoint said `src/cargo_free_self_build.rs` should remain dirty and uncommitted. Can the final clean Mantle status be claimed without contradicting that checkpoint?

Inspected evidence:

Command:

```sh
cd /home/brittonr/git/mantle
git status --short --branch
git status --short -- src/cargo_free_self_build.rs
git diff -- src/cargo_free_self_build.rs
git show --name-status --oneline --no-renames 556e3d45 -- src/cargo_free_self_build.rs
git show --name-status --oneline --no-renames 5301c64b -- src/cargo_free_self_build.rs
git log --oneline --name-status -- src/cargo_free_self_build.rs | sed -n '1,12p'
```

Output:

```text
## main...origin/main [ahead 3]
```

The `git status --short -- src/cargo_free_self_build.rs`, `git diff -- src/cargo_free_self_build.rs`, and both `git show ... -- src/cargo_free_self_build.rs` commands produced no output. The file-specific log output was:

```text
e33e1c8a Prepare native dynamic graph growth
M	src/cargo_free_self_build.rs
d56d1752 cli: make fixed-point proof reviewable
M	src/cargo_free_self_build.rs
61f773f5 make Cargo-free self-build a usable command
A	src/cargo_free_self_build.rs
```

Decision: The current Mantle worktree has no uncommitted `src/cargo_free_self_build.rs` change. The boundary commits `556e3d45` and `5301c64b` did not touch that file. Treat the current clean state as the inspected state for this review fix, and do not resurrect or invent missing dirty content.

Owner: Britton/user requested this review-fix checkpoint. The earlier owner decision remains historical for the prior dirty worktree; the current inspected tree no longer contains that dirty state.

Next action: If the old cargo-free self-build edits are still desired, recover them through the appropriate external source (for example reflog, stash, editor backup, or original branch) in a separate owner-requested follow-up. Do not include `src/cargo_free_self_build.rs` in Mantle/Onix boundary evidence unless a new request and current diff explicitly make it relevant.
