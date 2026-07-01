# Current Blocker — Project input retention roots

Date: 2026-07-01

## Question

Can `project-input-retention-roots` be honestly drained from the current tree?

## Inspected evidence

- `cairn/changes/project-input-retention-roots/tasks.md` still has 9 unchecked tasks covering retention policy schema, atomic update semantics, pure root action planning, shell persistence, diagnostics, and positive/negative/shell tests.
- The spec requires lock-generation binding, generation-limit retention based on Mantle-owned lock generation facts, source-state/root atomicity, and project diagnostics for pinned/unpinned/stale-root/missing-root/GC-eligible states.
- Current store/build code has GC roots for build/bootstrap/self-build outputs, but current project workflow code does not have a project input retention ledger, source-state root binding, or atomic lock/source-state/root transaction surface.
- `offline-source-bundle-manifest` is now archived, but this retention change still needs a dedicated retention ledger and atomic source-state/root update surface before imported source records can be honestly pinned.

## Decision

Blocked. Draining now would imply durable project-input retention and atomic source-state/root updates that the current project/store surfaces do not yet provide.

## Owner

Mantle project/store workflow owner after a project retention ledger design and atomic root persistence surface are available.

## Next action

1. define the lock-generation ledger and retention record identity;
2. implement pure policy validation and root action planning;
3. add shell persistence for roots and retention state with atomic commit/rollback diagnostics;
4. add diagnostics and tests for stale, missing, unpinned, and GC-eligible records.
