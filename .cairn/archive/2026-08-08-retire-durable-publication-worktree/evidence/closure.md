# Obsolete worktree closure

Date: 2026-08-08

## Observation

The finalizer fetched `origin` and proved that promotion archive commit `bd614a7fd6fa90a49e278cf5b683edc17fe518f6` is an ancestor of remote `main`.

The exact target was already absent:

```text
target_path=/home/brittonr/git/OnixResearch/.pi/worktrees/mantle-durable-publication
target_exists=false
target_registered=false
origin_main=6e670046c31fcc47169e857795c67a2b6a75bb69
promotion_archive_is_ancestor=true
```

The complete facts are in `target-facts.txt`. Their BLAKE3 digest is:

```text
61c4f848042ab53abe774239403faab76b49f020601863cb91d721bf43ac7dd8
```

`obsolete-target.ncl` binds the same facts with a typed Nickel contract.

## Decision

The change is obsolete. The exact-path requirement prevents substitution of the remaining promotion worktree or any other checkout. The proposed specification is not synchronized.

## Prohibited operations

The observation records:

```text
removal_attempted=false
recursive_delete_attempted=false
branch_delete_attempted=false
```

## Non-claims

This evidence does not claim that this change removed a worktree, pruned an administrative entry, deleted a branch, implemented a reusable eligibility core, or accepted a worktree-retirement capability. It proves only the recorded target absence, registration absence, and remote promotion ancestry.
