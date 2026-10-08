# Tasks: Add dynamic-plan source slices

Only evidenced tasks below are checked; proposal acceptance and archive remain
separate lifecycle decisions. The original late-unit regression exposed a signed
slice PathInfo before a V2 unit failed output-path admission. V2-only staged
unit, registry, and root-goal preflight now rejects that plan without publishing
the slice or changing live scheduler state; the passing regression and the
separate goal-capacity and named-CA-output cases are recorded in
`evidence/prepublication-repair-2026-10-03.md`. A current-source two-run CLI
producer/sandbox receipt also proves stable slice and unit identities when only
outside-slice bytes change. T4.4's exact before/after `--lib` and `--tests`,
strict current-source Clippy, and direct Cairn gates have complete scoped
receipts; the baseline and current Cargo.lock files differ, so this is not
identical-dependency A/B evidence. T4.5 is blocked pending an approved isolated
implementation commit, full dependency-source review, and lifecycle authorization;
there is no source-owner ACK for the shared tree's uncommitted V2 dependencies
and no accepted source snapshot.

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
- [x] [serial] T4.4 Run focused `crunch-build` lib and test suites before and after the change, strict Clippy for touched first-party packages, Cairn validation, and the proposal, design, and tasks gates. Preserve exact output in `evidence/`. r[mantle.dynamic_plan_source_slices.content_admission]
- [ ] [serial] T4.5 Sync accepted specs and archive through the isolated branch workflow with retained completion evidence. r[mantle.dynamic_plan_source_slices.provenance]
