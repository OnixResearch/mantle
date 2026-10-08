# Tasks: Adopt data-only dependency exports

All tasks remain open. Implementation is blocked behind the admission gate by
requirement.

## Phase 1: Admission gate (blocking)

- [ ] [serial] T1.1 Record the admission decision naming the first consumer surface, target outcome, adoption path, and maintenance owner, with re-review of this design against that surface. r[mantle.dependency_exports.bounded_adoption_gate]
- [ ] [serial] T1.2 Re-validate the exports record fields and registry options against the named consumer's real dependency shapes; amend the spec through this change if the review finds gaps. r[mantle.dependency_exports.exports_record]

## Phase 2: Contracts and evaluation (after admission)

- [ ] [serial] T2.1 Implement the exports record contract as a typed Nickel contract with tree-derived defaults, explicit overrides, placeholders, and bounds. r[mantle.dependency_exports.exports_record]
- [ ] [serial] T2.2 Implement registry and spec validation: unknown build system, unknown option, wrong option type, and unknown phase fail evaluation with named errors. r[mantle.dependency_exports.typed_build_systems]
- [ ] [parallel] T2.3 Add positive fixtures: record rendering of include, library, pkg-config, and environment paths from two dependencies. r[mantle.dependency_exports.exports_record]
- [ ] [parallel] T2.4 Add negative fixtures: hook-shaped field rejection, unknown option, wrong type, unknown phase, oversized record. r[mantle.dependency_exports.no_behavior_hooks] r[mantle.dependency_exports.typed_build_systems]

## Phase 3: Prepare rendering on the named consumer

- [ ] [serial] T3.1 Implement record-only prepare rendering for the admitted consumer family with no dependency code execution. r[mantle.dependency_exports.no_behavior_hooks]
- [ ] [serial] T3.2 Add an execution guard test proving no code from dependency outputs runs during prepare. r[mantle.dependency_exports.no_behavior_hooks]

## Phase 4: Verification

- [ ] [serial] T4.1 Run focused evaluation and shell tests before and after changes, strict Clippy, policy freshness, and relevant Nix checks. Preserve exact blockers. r[mantle.dependency_exports.typed_build_systems]
- [ ] [serial] T4.2 Sync accepted specs and archive through the isolated branch workflow with retained completion evidence. r[mantle.dependency_exports.bounded_adoption_gate]
