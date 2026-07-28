# Design

## Context

This change depends on `promote-durable-publication-adoption`. Cleanup starts only after that change is archived and its archive commit is present on Mantle remote `main`.

## Decisions

### Evaluate eligibility in a pure core

The core consumes expected path, clean status, registration, branch reachability, remote archive ancestry, unpushed commit count, and active-operation flags. The shell gathers facts and runs the approved command.

### Use a distinct finalizer checkout

The finalizer holds this cleanup change while the target worktree is removed. It must not reuse or modify the original dirty Mantle checkout.

### Remove one exact target

Use the version-control worktree removal and prune operations. Never use recursive deletion or branch deletion as fallback.

### Preserve unrelated state

Compare before-and-after manifests for the original checkout, all other worktrees, branches, and refs.

## Failure handling

Any dirty, untracked, unpublished, wrong-path, active-operation, remote-ancestry, or command failure leaves the target present and records the blocker.
