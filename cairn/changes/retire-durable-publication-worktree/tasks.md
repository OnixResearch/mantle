# Tasks

## Eligibility

- [ ] [serial] I1 Create a distinct finalizer checkout after the promotion archive commit is visible on Mantle remote `main`. r[mantle.worktree_retirement.prerequisites]
- [ ] [serial] I2 Gather bounded remote, branch, worktree, status, operation, and path facts for the exact target. r[mantle.worktree_retirement.eligibility]
- [ ] [parallel] V1 Add positive eligibility coverage and negative dirty, untracked, unpublished, wrong-path, and active-operation cases. r[mantle.worktree_retirement.eligibility]

## Retirement

- [ ] [serial] I3 Remove the exact target with the version-control worktree command and prune stale administrative entries. r[mantle.worktree_retirement.removal]
- [ ] [serial] V2 Verify the target is absent, branches and remote commits remain, and the original dirty checkout is unchanged. r[mantle.worktree_retirement.preservation]
- [ ] [parallel] V3 Record typed BLAKE3-bound cleanup evidence and explicit non-claims. r[mantle.worktree_retirement.evidence]
- [ ] [serial] V4 Run Cairn validation, gates, and focused traceability from the finalizer checkout. r[mantle.worktree_retirement.evidence]
- [ ] [serial] V5 Synchronize the accepted specification and archive the cleanup change from the finalizer checkout. r[mantle.worktree_retirement.evidence]
