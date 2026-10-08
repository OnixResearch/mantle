# Tasks: Add the spec override tree

All tasks remain open. Implementation is blocked behind the admission gate by
requirement.

## Phase 1: Admission gate (blocking)

- [ ] [serial] T1.1 Record the admission decision aligned with the package-layer family, naming the consumer surface, owner, and adoption path; re-review verb coverage against that surface. r[mantle.spec_override_tree.bounded_adoption_gate]

## Phase 2: Core semantics (after admission)

- [ ] [serial] T2.1 Implement pure tree parsing, verb application, and path validation over in-memory spec structures. r[mantle.spec_override_tree.verb_semantics] r[mantle.spec_override_tree.path_validation]
- [ ] [serial] T2.2 Implement layer merging with order preservation and final-set dependency name resolution. r[mantle.spec_override_tree.merge_cost_bound] r[mantle.spec_override_tree.path_validation]
- [ ] [parallel] T2.3 Add positive fixtures: every verb on its declared shape, order preservation across layers, dependency introduced by one layer and referenced by another. r[mantle.spec_override_tree.verb_semantics] r[mantle.spec_override_tree.merge_cost_bound]
- [ ] [parallel] T2.4 Add negative fixtures: unknown package, field missing without `set`, wrong-shape verb, unresolvable dependency, edited spec failing ordinary validation. r[mantle.spec_override_tree.path_validation]

## Phase 3: Evaluation integration

- [ ] [serial] T3.1 Integrate the merged tree with the package spec surface named at admission, with edited specs passing ordinary validation. r[mantle.spec_override_tree.verb_semantics]
- [ ] [serial] T3.2 Measure the layer-count bound over many layers and record the result. r[mantle.spec_override_tree.merge_cost_bound]

## Phase 4: Verification

- [ ] [serial] T4.1 Run focused core and evaluation tests before and after changes, strict Clippy, policy freshness, and relevant Nix checks. Preserve exact blockers. r[mantle.spec_override_tree.verb_semantics]
- [ ] [serial] T4.2 Sync accepted specs and archive through the isolated branch workflow with retained completion evidence. r[mantle.spec_override_tree.bounded_adoption_gate]
