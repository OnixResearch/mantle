# Tasks: Add dynamic-plan source slices

This isolated candidate is reconstructed on pinned `75ec931c8c5ae6e831d76ea12a93031f17e347f2`.
Receipts from the shared prepublication tree are historical and are not current-candidate
validation. Full candidate gates and archive acceptance remain unchecked until
clean-candidate gates actually pass. The CA unit-output placeholder fixture is owned
by `resolve-content-addressed-inputs-before-dispatch`, not this V2 change.

## Phase 1: Baseline and contract

- [x] [serial] T1.1 Record the baseline in an isolated worktree: accepted `mantle-plan-v1` golden bytes and digests, current source handling in `worker.rs` and `orchestrate.rs`, and focused `crunch-build` test output. r[mantle.dynamic_plan_source_slices.versioned_schema]
- [x] [serial] T1.2 Define the `mantle-plan-v2` wire and admitted types, slice grammar, named limits, and the stable rejection catalog. r[mantle.dynamic_plan_source_slices.versioned_schema] r[mantle.dynamic_plan_source_slices.bounded_rejection]
- [x] [serial] T1.3 Record the schema-version, slice-root, and content-identity decisions in an ADR with an index row in `adr/README.md`. r[mantle.dynamic_plan_source_slices.content_identity]

## Phase 2: Pure core

- [x] [serial] T2.1 Implement v2 decoding, admission, graph validation, canonical bytes, and plan digest beside the unchanged v1 path. r[mantle.dynamic_plan_source_slices.versioned_schema]
- [x] [serial] T2.2 Implement the pure admission planner over supplied tree facts with ordered output and typed rejections. r[mantle.dynamic_plan_source_slices.content_admission] r[mantle.dynamic_plan_source_slices.bounded_rejection]
- [x] [parallel] T2.3 Add positive core fixtures: two slices, identical-content deduplication, canonical ordering, and plan-digest coverage of slice digests. r[mantle.dynamic_plan_source_slices.content_identity]
- [x] [parallel] T2.4 Add negative core fixtures: absolute, `..`, empty-component, and over-long subpaths; an undeclared output; over-limit count and bytes; a conflicting duplicate id; v2 fields in a v1 document. r[mantle.dynamic_plan_source_slices.bounded_rejection] r[mantle.dynamic_plan_source_slices.versioned_schema]

## Phase 3: Worker and store integration

- [x] [serial] T3.1 Resolve slice subtrees from producer output nodes without following symlinks, verify observed digests, and publish slices through verified-source admission with signed PathInfo before unit registration. r[mantle.dynamic_plan_source_slices.content_admission]
- [x] [serial] T3.2 Make slice publication atomic per plan and keep registry, goal, scheduler, and success report state unchanged on any rejection, including a late unit/registry/goal failure before publication. r[mantle.dynamic_plan_source_slices.bounded_rejection]
- [x] [serial] T3.3 Add slice rows to native dynamic-plan reports and JSON build reports in canonical order. r[mantle.dynamic_plan_source_slices.provenance]
- [x] [parallel] T3.4 Add integration negative controls for an absent subpath, symlink traversal, a digest mismatch, and the admitted-byte limit, each proving unchanged scheduler state. r[mantle.dynamic_plan_source_slices.bounded_rejection]

## Phase 4: Verification and documentation

- [x] [serial] T4.1 Run a two-run fixture where the producer changes bytes outside one slice and prove unchanged slice store paths and unit derivation paths. r[mantle.dynamic_plan_source_slices.content_identity]
- [x] [serial] T4.2 Prove accepted `mantle-plan-v1` golden fixtures keep their canonical bytes and plan digests. r[mantle.dynamic_plan_source_slices.versioned_schema]
- [x] [serial] T4.3 Document `mantle-plan-v2` and slices in `docs/nominal-dynamic-plan-types.md`, including limits, rejection kinds, and non-claims. r[mantle.dynamic_plan_source_slices.provenance]
- [ ] [serial] T4.4 Run focused `crunch-build` lib and test suites before and after the change, strict Clippy for touched first-party packages, Cairn validation, and the proposal, design, and tasks gates. Preserve exact output in `evidence/`. r[mantle.dynamic_plan_source_slices.content_admission]
- [ ] [serial] T4.5 Sync accepted specs and archive through the isolated branch workflow with retained completion evidence. r[mantle.dynamic_plan_source_slices.provenance]
