## Phase 1: No-follow copy planning

- [ ] [serial] Define normalized no-follow tree entry observations and a pure deterministic copy planner with named path, entry, and depth bounds. r[mantle.release_provenance.bundle_tree_copy.plan]
- [ ] [serial] Reject invalid paths, parent shapes, duplicate entries, unsupported file kinds, and invalid bounds before mutation. r[mantle.release_provenance.bundle_tree_copy.plan.invalid]
- [ ] [serial] Define and test the supported relative in-tree symlink policy. r[mantle.release_provenance.bundle_tree_copy.symlink_policy] r[mantle.release_provenance.bundle_tree_copy.fixtures.negative.target]

## Phase 2: Capability-confined execution

- [ ] [serial] Replace following directory classification with no-follow enumeration that never recurses through symlinks. r[mantle.release_provenance.bundle_tree_copy.no_follow]
- [ ] [serial] Execute planned directory, file, and symlink operations relative to a trusted destination capability without following destination links. r[mantle.release_provenance.bundle_tree_copy.destination_confinement]
- [ ] [serial] Revalidate source entry and destination parent kinds at execution time and fail on type drift. r[mantle.release_provenance.bundle_tree_copy.fixtures.negative.type_drift]
- [ ] [serial] Make directory hashing and verification consume the same no-follow entry model and hash symlink target text without traversal. r[mantle.release_provenance.bundle_tree_copy.no_follow]

## Phase 3: Regression fixtures

- [ ] [parallel] Add positive nested-directory and supported internal-symlink copy/hash fixtures. r[mantle.release_provenance.bundle_tree_copy.fixtures.positive]
- [ ] [parallel] Add the audited directory-symlink exploit fixture and assert all external sentinel bytes and paths remain unchanged. r[mantle.release_provenance.bundle_tree_copy.fixtures.negative.symlink_escape]
- [ ] [parallel] Add source/destination symlink, absolute target, parent escape, type-swap, special-file, and bound-exhaustion negative fixtures. r[mantle.release_provenance.bundle_tree_copy.fixtures.negative.target] r[mantle.release_provenance.bundle_tree_copy.fixtures.negative.type_drift]

## Phase 4: Validation and documentation

- [ ] [serial] Document supported release-tree entry kinds, symlink restrictions, capability confinement, and bounded claims. r[mantle.release_provenance.bundle_tree_copy.symlink_policy] r[mantle.release_provenance.bundle_tree_copy.destination_confinement]
- [ ] [serial] Run focused release evidence unit/integration tests and the production external-sentinel regression matrix. r[mantle.release_provenance.bundle_tree_copy.validation] r[mantle.release_provenance.bundle_tree_copy.validation.production]
- [ ] [serial] Run Cairn validation and proposal, design, and tasks gates before sync/archive. r[mantle.release_provenance.bundle_tree_copy.validation]
