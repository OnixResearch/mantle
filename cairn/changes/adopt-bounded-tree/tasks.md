## Phase 1: Admit the shared dependency

- [ ] [serial] Confirm that `bounded-tree` has archived its establishment change and published a passing immutable Radicle revision. r[mantle.bounded_tree_adoption.prerequisite]
- [ ] [serial] Pin the reviewed Radicle revision in Mantle without a sibling path or GitHub fallback. r[mantle.bounded_tree_adoption.prerequisite]
- [ ] [parallel] Record the pre-adoption Mantle revision, dependency state, and rollback command. r[mantle.bounded_tree_adoption.rollback]

## Phase 2: Adapt release tree handling

- [ ] [serial] Map Mantle release observations, limits, blockers, and operations to `bounded-tree-core`. r[mantle.bounded_tree_adoption.release_copy]
- [ ] [serial] Replace local no-follow traversal, member hashing, revalidation, and copy effects with the shared capability shell. r[mantle.bounded_tree_adoption.release_copy]
- [ ] [parallel] Preserve Mantle release-root kinds, bundle layout, diagnostics, and evidence boundaries in the adapter. r[mantle.bounded_tree_adoption.boundary]

## Phase 3: Adapt frontend artifact handling

- [ ] [serial] Reuse shared ordered member facts in frontend artifact storage. r[mantle.bounded_tree_adoption.frontend_identity]
- [ ] [serial] Preserve the existing versioned frontend root preimage and executable-bit semantics. r[mantle.bounded_tree_adoption.frontend_identity]

## Phase 4: Prove parity and cut over

- [ ] [parallel] Dual-run valid release and frontend fixtures and compare plans, member facts, identities, and destination trees. r[mantle.bounded_tree_adoption.parity]
- [ ] [parallel] Dual-run malformed paths, limit failures, symlink failures, source changes, special files, and non-empty destinations. r[mantle.bounded_tree_adoption.parity]
- [ ] [serial] Remove only duplicated mechanism code after positive and negative parity passes. r[mantle.bounded_tree_adoption.parity]
- [ ] [serial] Run focused Cargo, Octet, Cairn, and release evidence gates before completing adoption. r[mantle.bounded_tree_adoption.boundary]
