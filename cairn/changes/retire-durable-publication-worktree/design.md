# Design

## Context

This change depends on `promote-durable-publication-adoption`. Its archive commit is present on Mantle remote `main`, but the exact cleanup target is already absent and unregistered.

## Decisions

### Keep the exact-path boundary

Do not substitute the remaining promotion worktree or any other checkout for `/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-durable-publication`.

### Close as an observed no-op

Record target absence, registration absence, remote ancestry, and prohibited-operation facts. Bind the facts file with BLAKE3 and a typed Nickel receipt.

### Do not synchronize the proposed specification

The one-time removal did not run. The proposed product requirements therefore do not become an accepted Mantle capability.

### Archive from a distinct finalizer

Move the obsolete change into the archive after Cairn validation. Preserve all worktrees, branches, refs, and product files.

## Failure handling

If the target reappears or remote ancestry does not hold, stop and leave the change active. Never use recursive deletion or a different path as fallback.
