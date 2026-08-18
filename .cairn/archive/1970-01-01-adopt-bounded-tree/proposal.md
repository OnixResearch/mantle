## Why

Mantle currently owns product-specific tree handling in release tree copying and frontend artifact storage. Those paths also contain the shared bounded traversal and planning mechanism that `bounded-tree` will own.

This change records a future cutover. It must remain unstarted until `bounded-tree` completes its release gates and publishes an immutable Radicle revision.

## What Changes

- Pin one reviewed `bounded-tree` Radicle revision without a sibling-path dependency.
- Replace local release tree admission and copy mechanics with the shared core and capability shell.
- Reuse shared member facts in frontend artifact storage while retaining Mantle's versioned root preimage.
- Preserve Mantle limits, diagnostics, release evidence meaning, and product policy through explicit adapters.
- Compare positive and negative fixtures before removing local mechanism code.
- Retain a rollback to the existing Mantle implementation until adoption evidence passes.

## Impact

- **Planned files**: `crates/crunch-release-core/src/tree_copy.rs`, `src/release_tree_copy.rs`, `src/frontend_artifact_store.rs`, Cargo and environment dependency declarations, focused tests, and release evidence.
- **Testing**: dual-run parity, malformed-tree and source-change failures, release bundle tests, frontend identity fixtures, Octet, Cargo, and Cairn gates.
- **Boundary**: Mantle retains build, store, artifact, evidence, and release semantics.
- **Current effect**: lifecycle planning only. No Mantle source or dependency changes are part of this task.
