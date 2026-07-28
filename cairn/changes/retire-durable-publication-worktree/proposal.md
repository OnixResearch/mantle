# Retire the Mantle durable publication worktree

## Why

The isolated Mantle adoption worktree must remain until canonical promotion is archived and remotely visible. This separate Cairn removes the lifecycle cycle from the promotion change.

## What Changes

- Continue from a separate finalizer checkout.
- Reject dirty, unpublished, wrong-path, and active-operation cases.
- Remove only `/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-durable-publication` through the version-control worktree command.
- Preserve Mantle's original dirty checkout and every unrelated worktree.

## Impact

This change removes one eligible local worktree. It does not delete branches, rewrite history, alter remote state, or touch the original dirty checkout.
