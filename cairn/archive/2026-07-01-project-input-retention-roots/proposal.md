# Proposal: Project input retention roots

## Summary

Add per-input retention policy so project inputs can be pinned across refreshes, branch switches, and rebases without relying on ad hoc store roots or accidental cache state.

## Motivation

Nixtamal's experimental GC tracking points at a real project workflow problem: lockfiles often need older inputs after a rebase or branch switch. Mantle already owns store and source state, so project input retention should be explicit, deterministic, and auditable.

Input retention should protect source/input material selected by a project policy while still making GC eligibility visible. It should not hide missing source facts or claim durability when no root exists.

## Scope

- Define project-level default retention and per-input overrides.
- Support tracked and untracked inputs, plus bounded generation retention for refreshed inputs.
- Bind retention roots to lockfile generations, source bundle imports, and generated input state where applicable.
- Report GC-eligible, pinned, stale-root, and missing-root states in project diagnostics.
- Make retention updates atomic with lock/source-state updates.

## Non-goals

- No background garbage collector.
- No global promise that remote caches keep old inputs.
- No silent retention of undeclared inputs or ambient package-manager caches.

## Target Spec Domains

- `project-workflows` for project input retention behavior.
- `store-transports` or source-state specs may later host lower-level archive/root mechanics if needed.
