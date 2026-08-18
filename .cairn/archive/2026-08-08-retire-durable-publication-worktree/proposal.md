# Retire the Mantle durable publication worktree

## Why

The named cleanup target was already absent and unregistered when the archived promotion became visible on remote `main`. Substituting a different worktree would violate the exact-path safety boundary.

## What Changes

- Record the remote prerequisite and exact target-absence facts from a separate finalizer checkout.
- Bind the observation to a BLAKE3 digest and a typed Nickel receipt.
- Archive this operational change as obsolete without synchronizing its proposed product requirements.
- Do not run worktree removal, recursive deletion, or branch deletion.

## Impact

This change records a no-op closure. It does not remove a worktree, delete a branch, rewrite history, alter product behavior, or make a cleanup-success claim.
