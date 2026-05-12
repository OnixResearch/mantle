## Why

The tracked root `.drain-state.md` still describes old active blockers even though the live OpenSpec queue is currently empty. Future ROI and drain sessions need a non-misleading handoff artifact.

## What Changes

- **Decide**: Decide whether `.drain-state.md` should be deleted, archived, or replaced with a current pointer to OpenSpec/spec status.
- **Preserve**: Preserve any still-useful facts in an appropriate durable doc or spec if deletion would lose context.
- **Verify**: Verify the repo/OpenSpec status after cleanup.

## Capabilities

### New Capabilities
- `bootstrap-state-handoff-hygiene`: Reconcile stale bootstrap drain state.

## Impact

- **Files**: `.drain-state.md`, potentially README or OpenSpec documentation.
- **APIs**: No public API change unless implementation tasks discover a necessary narrow seam.
- **Dependencies**: No new default dependency expected.
- **Testing**: Each task records the smallest relevant command or evidence artifact.
