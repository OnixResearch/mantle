## Phase 1: No-follow copy planning

- [x] [serial] Define normalized no-follow tree entry observations and a pure deterministic copy planner with named path, entry, and depth bounds. r[mantle.release_provenance.bundle_tree_copy.plan]
- [x] [serial] Reject invalid paths, parent shapes, duplicate entries, unsupported file kinds, and invalid bounds before mutation. r[mantle.release_provenance.bundle_tree_copy.plan.invalid]
- [x] [serial] Define and test the supported relative in-tree symlink policy. r[mantle.release_provenance.bundle_tree_copy.symlink_policy] r[mantle.release_provenance.bundle_tree_copy.fixtures.negative.target]

## Phase 2: Capability-confined execution

- [x] [serial] Replace following directory classification with no-follow enumeration that never recurses through symlinks. r[mantle.release_provenance.bundle_tree_copy.no_follow]
- [x] [serial] Execute planned directory, file, and symlink operations relative to a trusted destination capability without following destination links. r[mantle.release_provenance.bundle_tree_copy.destination_confinement]
- [x] [serial] Revalidate source entry and destination parent kinds at execution time and fail on type drift. r[mantle.release_provenance.bundle_tree_copy.fixtures.negative.type_drift]
- [x] [serial] Make directory hashing and verification consume the same no-follow entry model and hash symlink target text without traversal. r[mantle.release_provenance.bundle_tree_copy.no_follow]

## Phase 3: Regression fixtures

- [x] [parallel] Add positive nested-directory and supported internal-symlink copy/hash fixtures. r[mantle.release_provenance.bundle_tree_copy.fixtures.positive]
- [x] [parallel] Add the audited directory-symlink exploit fixture and assert all external sentinel bytes and paths remain unchanged. r[mantle.release_provenance.bundle_tree_copy.fixtures.negative.symlink_escape]
- [x] [parallel] Add source/destination symlink, absolute target, parent escape, type-swap, special-file, and bound-exhaustion negative fixtures. r[mantle.release_provenance.bundle_tree_copy.fixtures.negative.target] r[mantle.release_provenance.bundle_tree_copy.fixtures.negative.type_drift]

## Phase 4: Validation and documentation

- [x] [serial] Document supported release-tree entry kinds, symlink restrictions, capability confinement, and bounded claims. r[mantle.release_provenance.bundle_tree_copy.symlink_policy] r[mantle.release_provenance.bundle_tree_copy.destination_confinement]
- [x] [serial] Run focused release evidence unit/integration tests and the production external-sentinel regression matrix. r[mantle.release_provenance.bundle_tree_copy.validation] r[mantle.release_provenance.bundle_tree_copy.validation.production]
- [x] [serial] Run Cairn validation and proposal, design, and tasks gates before sync/archive. r[mantle.release_provenance.bundle_tree_copy.validation]

Current command evidence and the pre-change exploit baseline are recorded in [`evidence/validation.md`](evidence/validation.md).
