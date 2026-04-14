# Tasks: Remove host git from fetchGit

## Phase 1: Fetch implementation

- [ ] Delete arbitrary host `git` path discovery from `fetchGit`
- [ ] Implement a crunch-controlled `fetchGit` materialization path
- [ ] Keep the existing fetchGit user-facing contract intact

## Phase 2: Errors and tests

- [ ] Normalize invalid-revision and fetch failure errors around crunch-owned semantics
- [ ] Add tests that `fetchGit` no longer depends on host `PATH`
- [ ] Add tests that different host `git` availability does not change `fetchGit` behavior
- [ ] Run `openspec validate remove-host-git-from-fetchgit`
